extern crate arrayvec;
use self::arrayvec::ArrayVec;

extern crate rand;

extern crate htmf;

use rand::prelude::*;
use std::collections::HashMap;
use std::sync::atomic;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread::JoinHandle;

/**
 * Games are connected to each other via Moves.
 */
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Move {
    Place(u8),
    Move((u8, u8)),
}

/**
 * We keep one Tally per Game node, counting how many times we've tried a
 * given Move and how well that's worked out for us.
 */
#[derive(Clone)]
struct Tally {
    pub visits: HashMap<Move, (u64, f64)>,
    pub untried_moves: Vec<Move>,
}

impl Tally {
    pub fn new(game: &Game) -> Self {
        Self {
            visits: HashMap::new(),
            untried_moves: game.available_moves(),
        }
    }

    pub fn mark_visit(&mut self, edge: Move, reward: f64) {
        if let Some((visits, rewards)) = self.visits.get_mut(&edge) {
            *visits += 1;
            *rewards += reward;
        } else {
            self.visits.insert(edge.clone(), (1, reward));
        }
    }

    pub fn get_visit(&self, edge: Move) -> (u64, f64) {
        if let Some(&ans) = self.visits.get(&edge) {
            ans
        } else {
            (0, 0.0)
        }
    }

    pub fn merge(&mut self, other: Tally) {
        for (mov, (visits, rewards)) in other.visits.into_iter() {
            let (my_visits, my_rewards) = self.visits.entry(mov).or_insert((0, 0.0));
            *my_visits += visits;
            *my_rewards += rewards;
        }
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Game {
    state: htmf::game::GameState,
}

impl Game {
    fn current_player(&self) -> htmf::board::Player {
        self.state.active_player().unwrap()
    }

    fn available_moves(&self) -> Vec<Move> {
        if self.state.game_over() {
            return Vec::new();
        }
        let p = self.current_player();
        if self.state.finished_drafting() {
            let mut moves = Vec::new();
            for src in self.state.board.penguins[p.id].into_iter() {
                for dst in self.state.board.moves(src).into_iter() {
                    moves.push(Move::Move((src, dst)));
                }
            }
            moves
        } else {
            // Cells with one fish and nobody claiming them
            let mut draftable_cells = self.state.board.fish[0];
            draftable_cells.exclude(self.state.board.all_claimed_cells());
            draftable_cells.iter().map(Move::Place).collect()
        }
    }

    fn make_move(&mut self, mov: Move) {
        match mov {
            Move::Place(dst) => self.state.place_penguin(dst).unwrap(),
            Move::Move((src, dst)) => {
                let p = self.state.active_player().unwrap();
                self.state.board.claimed[p.id].insert(dst);
                self.state.board.penguins[p.id].remove(src);
                self.state.board.penguins[p.id].insert(dst);
                self.state.board.reap();
                self.state.turn += 1;
            }
        }
    }
}

fn choose_child(tally: &Tally, moves: &[Move], rng: &mut PolicyRng) -> Move {
    let total_visits = moves.iter().map(|&x| tally.get_visit(x).0).sum::<u64>();
    *rng.select_by_key(moves.iter(), |&&mov| {
        let (child_visits, sum_rewards) = tally.get_visit(mov);
        // https://www.researchgate.net/publication/235985858_A_Survey_of_Monte_Carlo_Tree_Search_Methods
        if child_visits == 0 {
            std::f64::INFINITY
        } else {
            let explore_term = (2.0 * (total_visits as f64).ln() / child_visits as f64).sqrt();
            let exploit_term = (sum_rewards + 1.0) / (child_visits as f64 + 2.0);
            explore_term + exploit_term
        }
    })
    .unwrap()
}

fn get_reward(game: &htmf::game::GameState, p: usize) -> f64 {
    let scores = game.get_scores();
    let winning_score = *scores.iter().max().unwrap();
    if scores[p] < winning_score {
        0.0
    } else if scores.iter().filter(|&&s| s >= winning_score).count() > 1 {
        0.5
    } else {
        1.0
    }
}

fn playout(root: &Game, tree: &mut HashMap<Game, Tally>, mut rng: &mut PolicyRng) {
    let mut path = vec![];
    let mut node = root.clone();
    while let Some(tally) = tree.get(&node) {
        if tally.untried_moves.is_empty() {
            break;
        }
        let mov = choose_child(tally, tally.untried_moves.as_slice(), &mut rng);
        path.push((node.clone(), mov));
        node.make_move(mov);
    }

    tree.entry(node.clone())
        .or_insert_with(|| Tally::new(&node));

    loop {
        let available_moves = node.available_moves();
        if available_moves.is_empty() {
            break;
        }
        let &mov = available_moves.choose(&mut rng.rng).unwrap();
        path.push((node.clone(), mov));
        node.make_move(mov);
    }

    assert!(node.state.game_over());
    let rewards: ArrayVec<[f64; 4]> = (0..root.state.nplayers)
        .map(|p| get_reward(&node.state, p))
        .collect();
    for (backprop_node, mov) in path {
        let p = backprop_node.state.active_player().unwrap();
        if let Some(tally) = tree.get_mut(&backprop_node) {
            tally.mark_visit(mov, rewards[p.id]);
        } else {
            break;
        }
    }
}

pub struct MCTSBot {
    pub root: Game,
    pub me: htmf::board::Player,
    tree: HashMap<Game, Tally>,
    rng: PolicyRng,
    ponderer: Option<Ponderer>,
}

impl MCTSBot {
    pub fn new(game: htmf::game::GameState, me: htmf::board::Player) -> Self {
        MCTSBot {
            root: Game { state: game },
            me,
            tree: HashMap::new(),
            rng: PolicyRng::default(),
            ponderer: None,
        }
    }

    pub fn update(&mut self, game: htmf::game::GameState) {
        self.tree.retain(|g, _| g.state.turn >= game.turn);
        self.root = Game { state: game };
        self.finish_pondering();
    }

    /// Spawn a background thread to process new moves
    pub fn ponder(&mut self) {
        self.ponderer = Some(Ponderer::new(self.root.clone(), self.tree.clone()));
    }

    /// Join the background thread and process the work it did
    pub fn finish_pondering(&mut self) {
        if let Some(ponder_tree) = self.ponderer.take().map(|p| p.finish()) {
            println!("Processing {} ponders", ponder_tree.len());
            for (game, new_tally) in ponder_tree.into_iter() {
                self.tree
                    .entry(game.clone())
                    .or_insert_with(|| Tally::new(&game))
                    .merge(new_tally);
            }
        }
    }

    fn num_playouts(&self) -> usize {
        10 * self.root.available_moves().len()
    }

    pub fn take_action(&mut self) -> htmf::game::Action {
        if self.root.state.active_player() != Some(self.me) {
            panic!("{:?} was asked to move, but it is not their turn!", self.me);
        }
        self.finish_pondering();
        let nplayouts = self.num_playouts();
        for _ in 0..nplayouts {
            playout(&self.root, &mut self.tree, &mut self.rng);
        }
        let tally = self.tree.get(&self.root).unwrap();
        let (best_move, (_visits, _reward)) = tally
            .visits
            .iter()
            .max_by(|(_, &(visits1, score1)), (_, &(visits2, score2))| {
                (score1 / visits1 as f64)
                    .partial_cmp(&(score2 / visits2 as f64))
                    .unwrap()
            })
            .unwrap();
        match *best_move {
            Move::Move((src, dst)) => htmf::game::Action::Move(src, dst),
            Move::Place(dst) => htmf::game::Action::Place(dst),
        }
    }
}

struct Ponderer {
    thread: Option<JoinHandle<HashMap<Game, Tally>>>,
    should_run: Arc<AtomicBool>,
}

impl Ponderer {
    pub fn new(game: Game, mut tree: HashMap<Game, Tally>) -> Self {
        let should_run = Arc::new(AtomicBool::new(true));
        let should_run2 = should_run.clone();
        Self {
            thread: Some(std::thread::spawn(move || {
                let mut rng = PolicyRng::default();
                while should_run2.load(atomic::Ordering::Relaxed) {
                    playout(&game, &mut tree, &mut rng);
                }
                tree
            })),
            should_run,
        }
    }

    pub fn finish(mut self) -> HashMap<Game, Tally> {
        self.should_run.store(false, atomic::Ordering::SeqCst);
        self.thread.take().unwrap().join().unwrap()
    }
}

impl Drop for Ponderer {
    fn drop(&mut self) {
        self.should_run.store(false, atomic::Ordering::Relaxed);
    }
}

#[derive(Clone)]
struct PolicyRng {
    rng: ThreadRng,
}

impl PolicyRng {
    pub fn select_by_key<T, Iter, KeyFn>(&mut self, elts: Iter, mut key_fn: KeyFn) -> Option<T>
    where
        Iter: Iterator<Item = T>,
        KeyFn: FnMut(&T) -> f64,
    {
        let mut choice = None;
        let mut num_optimal: u32 = 0;
        let mut best_so_far: f64 = std::f64::NEG_INFINITY;
        for elt in elts {
            let score = key_fn(&elt);
            if score > best_so_far {
                choice = Some(elt);
                num_optimal = 1;
                best_so_far = score;
            } else if (score - best_so_far).abs() < std::f64::EPSILON {
                num_optimal += 1;
                if self.rng.gen_bool(1.0 / num_optimal as f64) {
                    choice = Some(elt);
                }
            }
        }
        choice
    }
}

impl Default for PolicyRng {
    fn default() -> Self {
        Self { rng: thread_rng() }
    }
}

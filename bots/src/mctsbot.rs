use arrayvec::ArrayVec;
use rand::prelude::*;
use std::collections::HashMap;

use htmf::{NUM_ONE_FISH, NUM_THREE_FISH, NUM_TWO_FISH};

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
#[derive(Clone, Default)]
pub struct Tally {
    pub visits: HashMap<Move, (u64, f64)>,
}

impl Tally {
    pub fn new(game: &Game) -> Self {
        Self {
            visits: game.available_moves().map(|mov| (mov, (0, 0.0))).collect(),
        }
    }

    pub fn mark_visit(&mut self, edge: Move, reward: f64) {
        let (ref mut visits, ref mut rewards) = self.visits.entry(edge).or_insert((0, 0.0));
        *visits += 1;
        *rewards += reward;
    }

    pub fn get_visit(&self, edge: Move) -> (u64, f64) {
        if let Some(&ans) = self.visits.get(&edge) {
            ans
        } else {
            (0, 0.0)
        }
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Game {
    pub state: htmf::game::GameState,
}

impl Game {
    fn current_player(&self) -> htmf::board::Player {
        self.state.active_player().unwrap()
    }

    fn is_won(&self) -> bool {
        let total_fish = NUM_ONE_FISH + 2 * NUM_TWO_FISH + 3 * NUM_THREE_FISH;
        self.state.get_scores().into_iter().max().unwrap() > total_fish / self.state.nplayers
    }

    fn available_moves<'a>(&'a self) -> Box<dyn Iterator<Item = Move> + 'a> {
        if self.state.game_over() {
            Box::new(std::iter::empty())
        } else if self.state.finished_drafting() {
            let p = self.current_player();
            Box::new(
                self.state.board.penguins[p.id]
                    .into_iter()
                    .flat_map(move |src| {
                        self.state
                            .board
                            .moves(src)
                            .into_iter()
                            .map(move |dst| Move::Move((src, dst)))
                    }),
            )
        } else {
            // Cells with one fish and nobody claiming them
            let draftable_cells =
                self.state.board.fish[0].exclude(self.state.board.all_claimed_cells());
            Box::new(draftable_cells.into_iter().map(Move::Place))
        }
    }

    fn make_move(&mut self, mov: Move) {
        match mov {
            Move::Place(dst) => self.state.place_penguin(dst).unwrap(),
            Move::Move((src, dst)) => {
                let p = self.state.active_player().unwrap();
                self.state.board.claimed[p.id] = self.state.board.claimed[p.id].insert(dst);
                self.state.board.penguins[p.id] = self.state.board.penguins[p.id].remove(src);
                self.state.board.penguins[p.id] = self.state.board.penguins[p.id].insert(dst);
                self.state.board.reap();
                self.state.turn += 1;
            }
        }
    }
}

fn choose_child(tally: &Tally, rng: &mut impl rand::Rng) -> Move {
    let total_visits = tally
        .visits
        .iter()
        .map(|(_, (visits, _))| visits)
        .sum::<u64>();

    let mut choice = None;
    let mut num_optimal: u32 = 0;
    let mut best_so_far: f64 = std::f64::NEG_INFINITY;
    for (&mov, &(child_visits, sum_rewards)) in tally.visits.iter() {
        let score = {
            // https://www.researchgate.net/publication/235985858_A_Survey_of_Monte_Carlo_Tree_Search_Methods
            if child_visits == 0 {
                std::f64::INFINITY
            } else {
                let explore_term = (2.0 * (total_visits as f64).ln() / child_visits as f64).sqrt();
                let exploit_term = (sum_rewards + 1.0) / (child_visits as f64 + 2.0);
                explore_term + exploit_term
            }
        };
        if score > best_so_far {
            choice = Some(mov);
            num_optimal = 1;
            best_so_far = score;
        } else if (score - best_so_far).abs() < std::f64::EPSILON {
            num_optimal += 1;
            if rng.gen_bool(1.0 / num_optimal as f64) {
                choice = Some(mov);
            }
        }
    }
    choice.unwrap()
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

fn playout(root: &Game, tree: &mut HashMap<Game, Tally>, mut rng: &mut ThreadRng) {
    let mut path = vec![];
    let mut node = root.clone();
    while let Some(tally) = tree.get(&node) {
        if tally.visits.is_empty() {
            break;
        }
        let mov = choose_child(tally, &mut rng);
        path.push((node.clone(), mov));
        node.make_move(mov);
    }

    tree.entry(node.clone())
        .or_insert_with(|| Tally::new(&node));

    loop {
        if node.is_won() {
            break;
        }
        if let Some(mov) = node.available_moves().choose(&mut rng) {
            path.push((node.clone(), mov));
            node.make_move(mov);
        } else {
            break;
        }
    }

    assert!(path[0].0 == *root);
    let rewards: ArrayVec<f64, 4> = (0..root.state.nplayers)
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
    assert!(tree.get(root).is_some())
}

/// Move nodes from `old_tree` to `new_tree` if they're reachable from `game`
fn move_reachable_nodes(
    new_tree: &mut HashMap<Game, Tally>,
    old_tree: &mut HashMap<Game, Tally>,
    game: Game,
) {
    let mut tally_to_add = Tally::default();
    if let Some(tally) = old_tree.get_mut(&game) {
        std::mem::swap(tally, &mut tally_to_add);
    } else {
        return;
    }

    for (mov, _) in tally_to_add.visits.iter() {
        let mut new_game = game.clone();
        new_game.make_move(*mov);
        move_reachable_nodes(new_tree, old_tree, new_game);
    }
    new_tree.insert(game, tally_to_add);
}

#[derive(Clone)]
pub struct MCTSBot {
    pub root: Game,
    pub me: htmf::board::Player,
    pub tree: HashMap<Game, Tally>,
    rng: ThreadRng,
}

impl MCTSBot {
    pub fn new(game: htmf::game::GameState, me: htmf::board::Player) -> Self {
        MCTSBot {
            root: Game { state: game },
            me,
            tree: HashMap::new(),
            rng: thread_rng(),
        }
    }

    /// Tell the bot about the new game state
    pub fn update(&mut self, game: htmf::game::GameState) {
        self.root = Game { state: game };
        self.prune();
    }

    /// Keep only entries in self.tree that are reachable from self.root
    fn prune(&mut self) {
        let mut new_tree = HashMap::new();
        move_reachable_nodes(&mut new_tree, &mut self.tree, self.root.clone());
        self.tree = new_tree;
    }

    pub fn playout(&mut self) {
        playout(&self.root, &mut self.tree, &mut self.rng)
    }

    pub fn take_action(&mut self) -> htmf::game::Action {
        if self.root.state.active_player() != Some(self.me) {
            panic!("{:?} was asked to move, but it is not their turn!", self.me);
        }
        playout(&self.root, &mut self.tree, &mut self.rng);
        let tally = self.tree.get(&self.root).unwrap();
        let best_move = *tally
            .visits
            .iter()
            .max_by(|(_, &(visits1, score1)), (_, &(visits2, score2))| {
                (score1 / visits1 as f64)
                    .partial_cmp(&(score2 / visits2 as f64))
                    .unwrap_or(visits1.cmp(&visits2))
            })
            .unwrap()
            .0;
        match best_move {
            Move::Move((src, dst)) => htmf::game::Action::Move(src, dst),
            Move::Place(dst) => htmf::game::Action::Place(dst),
        }
    }

    pub fn tree_size(&self) -> usize {
        self.tree.len()
    }
}

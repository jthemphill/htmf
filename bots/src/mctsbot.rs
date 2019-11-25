extern crate rand;

extern crate htmf;

use rand::prelude::*;
use std::collections::HashMap;

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
}

impl Tally {
    pub fn mark_visit(&mut self, edge: &Move, reward: f64) {
        if let Some((visits, rewards)) = self.visits.get_mut(edge) {
            *visits += 1;
            *rewards += reward;
        } else {
            self.visits.insert(edge.clone(), (1, reward));
        }
    }

    pub fn get_visit(&self, edge: &Move) -> (u64, f64) {
        if let Some(ans) = self.visits.get(edge) {
            ans.clone()
        } else {
            (0, 0.0)
        }
    }
}

impl Tally {
    pub fn new() -> Self {
        Tally {
            visits: HashMap::new(),
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
            let mut draftable_cells = self.state.board.fish[0].clone();
            draftable_cells.exclude(&self.state.board.all_claimed_cells());
            draftable_cells
                .iter()
                .map(|cell| Move::Place(cell))
                .collect()
        }
    }

    fn make_move(&mut self, mov: &Move) {
        match mov {
            Move::Place(dst) => self.state.place_penguin(*dst).unwrap(),
            Move::Move((src, dst)) => {
                let p = self.state.active_player().unwrap();
                self.state.board.claimed[p.id].insert(*dst);
                self.state.board.penguins[p.id].remove(*src);
                self.state.board.penguins[p.id].insert(*dst);
            }
        }
    }
}

fn choose_child(tally: &Tally, moves: &Vec<Move>, rng: &mut PolicyRng) -> Move {
    let exploration_constant = 0.5;

    let total_visits = moves.iter().map(|x| tally.get_visit(x).0).sum::<u64>();
    let adjusted_total = (total_visits + 1) as f64;
    let ln_adjusted_total = adjusted_total.ln();
    rng.select_by_key(moves.iter(), |mov| {
        let (child_visits, sum_rewards) = tally.get_visit(mov);
        // http://mcts.ai/pubs/mcts-survey-master.pdf
        let explore_term = if child_visits == 0 {
            std::f64::INFINITY
        } else {
            2.0 * (ln_adjusted_total / child_visits as f64).sqrt()
        };
        let mean_action_value = sum_rewards as f64 / adjusted_total;
        exploration_constant * explore_term + mean_action_value
    })
    .unwrap()
    .clone()
}

#[derive(Clone)]
pub struct MCTSBot {
    pub root: Game,
    pub me: htmf::board::Player,
    rng: PolicyRng,
    tree: HashMap<Game, Tally>,
}

impl MCTSBot {
    pub fn new(game: htmf::game::GameState, me: htmf::board::Player) -> Self {
        MCTSBot {
            root: Game { state: game },
            me,
            rng: PolicyRng::default(),
            tree: HashMap::new(),
        }
    }

    pub fn update(&mut self, game: htmf::game::GameState) {
        self.root = Game { state: game };
    }

    pub fn take_action(&mut self) -> htmf::game::Action {
        if self.root.state.active_player() != Some(self.me) {
            panic!("{:?} was asked to move, but it is not their turn!", self.me);
        }
        for _ in 0..50 {
            self.playout();
        }
        let tally = self.tree.get(&self.root).unwrap();
        let best_move = *tally
            .visits
            .iter()
            .max_by(|(_, score1), (_, score2)| score1.partial_cmp(score2).unwrap())
            .map(|(mov, _)| mov)
            .unwrap();
        match best_move {
            Move::Move((src, dst)) => htmf::game::Action::Move(src, dst),
            Move::Place(dst) => htmf::game::Action::Place(dst),
        }
    }

    fn playout(&mut self) {
        let mut path = vec![];
        let mut node = self.root.clone();
        loop {
            let moves = node.available_moves();
            if moves.len() == 0 {
                break;
            }
            let tally = if let Some(tally) = self.tree.get(&node) {
                tally
            } else {
                self.tree.insert(node.clone(), Tally::new());
                self.tree.get(&node).unwrap()
            };
            let mov = choose_child(tally, &moves, &mut self.rng);
            path.push(mov);
            node.make_move(&mov);
            if self.tree.get(&node).is_none() {
                self.tree.insert(node.clone(), Tally::new());
                break;
            }
        }
        let scores = node.state.get_scores();
        let reward = scores[self.me.id] as f64;
        let mut backprop_node = self.root.clone();
        for mov in path {
            let p = node.state.active_player().unwrap();
            let reward = if p == self.me { reward } else { -reward };
            self.tree
                .get_mut(&backprop_node)
                .unwrap()
                .mark_visit(&mov, reward);
            backprop_node.make_move(&mov);
        }
    }
}

#[derive(Clone)]
pub struct PolicyRng {
    rng: StdRng,
}

impl PolicyRng {
    pub fn new() -> Self {
        let mut seed = [1; 32];
        for i in 1..32 {
            seed[i] = i as u8;
        }
        let rng = SeedableRng::from_seed(seed);
        Self { rng }
    }

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
            } else if score == best_so_far {
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
        Self::new()
    }
}

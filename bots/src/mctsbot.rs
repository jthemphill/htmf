use arrayvec::ArrayVec;
use rand::prelude::*;

/**
 * Games are connected to each other via Moves.
 */
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Move {
    Place(u8),
    Move((u8, u8)),
}

impl From<htmf::game::Action> for Move {
    fn from(action: htmf::game::Action) -> Self {
        match action {
            htmf::game::Action::Move(src, dst) => Self::Move((src, dst)),
            htmf::game::Action::Place(dst) => Self::Place(dst),
            _ => panic!("Can't convert {:?} into a move or place", action),
        }
    }
}

impl Into<htmf::game::Action> for Move {
    fn into(self) -> htmf::game::Action {
        match self {
            Move::Move((src, dst)) => htmf::game::Action::Move(src, dst),
            Move::Place(dst) => htmf::game::Action::Place(dst),
        }
    }
}

pub struct TreeNode {
    pub game: Game,
    pub visits: u64,
    pub rewards: f64,
    pub children: Option<Vec<(Move, TreeNode)>>,
}

impl TreeNode {
    pub fn new(game: Game) -> Self {
        Self {
            game,
            visits: 0,
            rewards: 0.0,
            children: Option::None,
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.children.is_none()
    }

    pub fn mark_visit(&mut self, reward: f64) {
        self.visits += 1;
        self.rewards += reward;
    }

    pub fn get_mut_child(&mut self, game_move: Move) -> &mut TreeNode {
        &mut self
            .iter_mut_children()
            .find(|(child_move, _)| *child_move == game_move)
            .unwrap()
            .1
    }

    pub fn iter_mut_children(&mut self) -> impl Iterator<Item = &mut (Move, TreeNode)> {
        let children = self.children.get_or_insert_with(|| {
            self.game
                .available_moves()
                .map(|child_move| {
                    let mut game = self.game.clone();
                    game.make_move(child_move);
                    (child_move, TreeNode::new(game))
                })
                .collect()
        });
        children.iter_mut()
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

fn choose_child<'tree, R: Rng + ?Sized>(
    node: &'tree mut TreeNode,
    rng: &'_ mut R,
) -> (Move, &'tree mut TreeNode) {
    let total_visits = node.visits;

    let mut chosen_child = None;
    let mut num_optimal: u32 = 0;
    let mut best_so_far: f64 = std::f64::NEG_INFINITY;
    for (child_move, child) in node.iter_mut_children() {
        let score = {
            // https://www.researchgate.net/publication/235985858_A_Survey_of_Monte_Carlo_Tree_Search_Methods
            if child.visits == 0 {
                std::f64::INFINITY
            } else {
                let explore_term = (2.0 * (total_visits as f64).ln() / child.visits as f64).sqrt();
                let exploit_term = (child.rewards + 1.0) / (child.visits as f64 + 2.0);
                explore_term + exploit_term
            }
        };
        if score > best_so_far {
            chosen_child = Some((*child_move, child));
            num_optimal = 1;
            best_so_far = score;
        } else if (score - best_so_far).abs() < std::f64::EPSILON {
            num_optimal += 1;
            if rng.gen_bool(1.0 / num_optimal as f64) {
                chosen_child = Some((*child_move, child));
            }
        }
    }
    chosen_child.unwrap()
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

fn playout<R: Rng + ?Sized>(root: &mut TreeNode, rng: &mut R) -> (Vec<Move>, Game) {
    let mut path = vec![];
    let mut expand_node = root;

    // Find a leaf node
    while !expand_node.is_leaf() {
        let (child_move, child_node) = choose_child(expand_node, rng);
        expand_node = child_node;
        path.push(child_move);
    }

    // Expand the tree by creating one more node
    if !expand_node.game.state.game_over() {
        let (child_move, child_node) = choose_child(expand_node, rng);
        expand_node = child_node;
        path.push(child_move);
    }

    // Finish the game
    let mut game = expand_node.game.clone();
    while let Some(game_move) = game.available_moves().choose(rng) {
        path.push(game_move);
        game.make_move(game_move);
    }

    (path, game)
}

fn backprop(root: &mut TreeNode, path: Vec<Move>, game: Game) {
    root.visits += 1;
    let rewards: ArrayVec<f64, 4> = (0..game.state.nplayers)
        .map(|p| get_reward(&game.state, p))
        .collect();
    let mut backprop_node = root;
    for backprop_move in path {
        if backprop_node.is_leaf() {
            break;
        }
        if let Some(p) = backprop_node.game.state.active_player() {
            backprop_node = backprop_node.get_mut_child(backprop_move);
            backprop_node.mark_visit(rewards[p.id]);
        } else {
            break;
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct UpdateStats {
    pub old_size: usize,
    pub old_capacity: usize,
    pub new_size: usize,
    pub new_capacity: usize,
}

pub struct MCTSBot<R: Rng> {
    pub root: TreeNode,
    pub me: htmf::board::Player,
    rng: R,
}

impl<R: Rng> MCTSBot<R> {
    pub fn new(game: htmf::game::GameState, me: htmf::board::Player, rng: R) -> Self {
        MCTSBot {
            root: TreeNode::new(Game { state: game }),
            me,
            rng,
        }
    }

    /// Tell the bot about the new game state
    pub fn update(&mut self, game_state: &htmf::game::GameState) {
        let dummy = TreeNode {
            game: Game {
                state: game_state.clone(),
            },
            children: None,
            rewards: 0.0,
            visits: 0,
        };
        let chosen_edge = self
            .root
            .iter_mut_children()
            .find(|(_, child)| child.game.state == *game_state);
        if let Some((_, chosen_child)) = chosen_edge {
            self.root = std::mem::replace(chosen_child, dummy);
        } else {
            self.root = dummy;
        }
    }

    pub fn playout(&mut self) {
        let (path, game) = playout(&mut self.root, &mut self.rng);
        backprop(&mut self.root, path, game);
    }

    pub fn take_action(&mut self) -> htmf::game::Action {
        if self.root.game.state.active_player() != Some(self.me) {
            panic!("{:?} was asked to move, but it is not their turn!", self.me);
        }
        playout(&mut self.root, &mut self.rng);
        let (best_move, _) = self
            .root
            .iter_mut_children()
            .max_by(|(_, child1), (_, child2)| {
                (child1.rewards / child1.visits as f64)
                    .partial_cmp(&(child2.rewards / child2.visits as f64))
                    .unwrap_or(child1.visits.cmp(&child2.visits))
            })
            .unwrap();
        match *best_move {
            Move::Move((src, dst)) => htmf::game::Action::Move(src, dst),
            Move::Place(dst) => htmf::game::Action::Place(dst),
        }
    }

    pub fn tree_size(&self) -> usize {
        let mut stack = vec![&self.root];
        let mut sum = 0;
        while let Some(node) = stack.pop() {
            sum += 1;
            if let Some(children) = &node.children {
                for (_, child) in children {
                    stack.push(child);
                }
            }
        }
        sum
    }
}

#[test]
fn test_run_full_game() {
    use htmf::board::Player;
    use htmf::game::GameState;

    let mut game = GameState::new_two_player::<StdRng>(&mut SeedableRng::seed_from_u64(0));
    let mut bots = (0..=1)
        .map(|i| {
            MCTSBot::new(
                game.clone(),
                Player { id: i },
                SeedableRng::seed_from_u64(i as u64),
            )
        })
        .collect::<Vec<MCTSBot<StdRng>>>();

    while let Some(p) = game.active_player() {
        let action = bots[p.id].take_action();
        game.apply_action(&action).unwrap();
        for bot in &mut bots {
            bot.update(&game);
        }
    }
}

use rand::prelude::*;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::OnceLock;

const NUM_PLAYERS: usize = 2;

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
    // u64 representation of (rewards: f32, visits: u32)
    pub rewards_visits: RewardsVisits,
    pub children: OnceLock<Vec<(Move, TreeNode)>>,
}

impl TreeNode {
    pub fn new() -> Self {
        Self {
            rewards_visits: Default::default(),
            children: Default::default(),
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.children.get().is_none()
    }

    pub fn get_child(&self, game_move: Move) -> &TreeNode {
        &self
            .children
            .get()
            .unwrap()
            .iter()
            .find(|(child_move, _)| *child_move == game_move)
            .unwrap()
            .1
    }

    pub fn iter_children<'a>(
        &'a self,
        game: &Game,
        node_count: Option<&AtomicUsize>,
    ) -> impl Iterator<Item = &'a (Move, TreeNode)> {
        self.children
            .get_or_init(|| {
                let children: Vec<_> = game
                    .available_moves()
                    .map(|child_move| (child_move, TreeNode::new()))
                    .collect();

                if let Some(node_count) = node_count {
                    node_count.fetch_add(children.len(), Ordering::Relaxed);
                }

                children
            })
            .iter()
    }

    pub fn iter_mut_children<'a>(
        &'a mut self,
        game: &Game,
        node_count: Option<&AtomicUsize>,
    ) -> impl Iterator<Item = &'a mut (Move, TreeNode)> {
        self.children.get_or_init(|| {
            let children: Vec<_> = game
                .available_moves()
                .map(|child_move| (child_move, TreeNode::new()))
                .collect();

            if let Some(node_count) = node_count {
                node_count.fetch_add(children.len(), Ordering::Relaxed);
            }

            children
        });
        self.children.get_mut().unwrap().iter_mut()
    }
}

#[derive(Default, Debug)]
pub struct RewardsVisits {
    pub rewards_visits: AtomicU64,
}

impl RewardsVisits {
    pub fn get(&self) -> (f32, u32) {
        Self::decompose_rewards_visits(self.rewards_visits.load(Ordering::Relaxed))
    }

    pub fn get_and_increment_visits(&self) -> (f32, u32) {
        Self::decompose_rewards_visits(self.rewards_visits.fetch_add(1, Ordering::Relaxed))
    }

    pub fn add_reward(&self, reward: f32) {
        self.rewards_visits
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |rewards_visits| {
                let (mut rewards, visits) = Self::decompose_rewards_visits(rewards_visits);
                rewards += reward;
                Some(((rewards.to_bits() as u64) << 32) | (visits as u64))
            })
            .unwrap();
    }

    fn decompose_rewards_visits(rewards_visits: u64) -> (f32, u32) {
        let rewards = f32::from_bits((rewards_visits >> 32) as u32);
        let visits = rewards_visits as u32;
        (rewards, visits)
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
    node: &'tree TreeNode,
    game: &Game,
    rng: &'_ mut R,
    node_count: &AtomicUsize,
) -> (Move, &'tree TreeNode) {
    // Increment visits now to discourage other threads from following this path through the tree
    let (_, total_visits) = node.rewards_visits.get_and_increment_visits();

    let mut chosen_child = None;
    let mut num_optimal: f64 = 0.0;
    let mut best_so_far: f32 = std::f32::NEG_INFINITY;
    for (child_move, child) in node.iter_children(game, Some(node_count)) {
        let score = {
            let (child_rewards, child_visits) = child.rewards_visits.get();
            // https://www.researchgate.net/publication/235985858_A_Survey_of_Monte_Carlo_Tree_Search_Methods
            if child_visits == 0 {
                std::f32::INFINITY
            } else {
                let explore_term = (2.0 * (total_visits as f32).ln() / child_visits as f32).sqrt();
                let exploit_term = child_rewards / child_visits as f32;
                explore_term + exploit_term
            }
        };
        if score > best_so_far {
            chosen_child = Some((*child_move, child));
            num_optimal = 1.0;
            best_so_far = score;
        } else if (score - best_so_far).abs() < std::f32::EPSILON {
            num_optimal += 1.0;
            if rng.random_bool(1.0 / num_optimal) {
                chosen_child = Some((*child_move, child));
            }
        }
    }
    chosen_child.unwrap()
}

fn get_reward(game: &htmf::game::GameState, p: usize) -> f32 {
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

fn playout(root: &TreeNode, root_game: &Game, node_count: &AtomicUsize) -> (Vec<Move>, Game) {
    let rng = &mut rand::rng();
    let mut path = vec![];
    let mut expand_node = root;
    let mut game = root_game.clone();

    // Find a leaf node
    while !expand_node.is_leaf() {
        let (child_move, child_node) = choose_child(expand_node, &game, rng, node_count);
        game.make_move(child_move);
        expand_node = child_node;
        path.push(child_move);
    }

    // Expand the tree by creating one more node
    if !game.state.game_over() {
        let (child_move, _child_node) = choose_child(expand_node, &game, rng, node_count);
        game.make_move(child_move);
        path.push(child_move);
    }

    // Finish the game
    while let Some(game_move) = game.available_moves().choose(rng) {
        path.push(game_move);
        game.make_move(game_move);
    }

    (path, game)
}

fn backprop(root: &TreeNode, root_game: &Game, path: Vec<Move>, game: Game) {
    let rewards: [f32; NUM_PLAYERS] = [get_reward(&game.state, 0), get_reward(&game.state, 1)];
    let mut backprop_node = root;
    let mut current_game = root_game.clone();

    for backprop_move in path {
        if backprop_node.is_leaf() {
            break;
        }
        if let Some(p) = current_game.state.active_player() {
            backprop_node = backprop_node.get_child(backprop_move);
            backprop_node.rewards_visits.add_reward(rewards[p.id]);
            current_game.make_move(backprop_move);
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

pub struct MCTSBot {
    pub root: TreeNode,
    pub root_game: Game,
    pub me: htmf::board::Player,
    pub num_nodes: AtomicUsize,
}

impl MCTSBot {
    pub fn new(game: htmf::game::GameState, me: htmf::board::Player) -> Self {
        MCTSBot {
            root: TreeNode::new(),
            root_game: Game { state: game },
            me,
            num_nodes: AtomicUsize::new(1),
        }
    }

    /// Tell the bot about the new game state
    pub fn update(&mut self, game_state: &htmf::game::GameState) {
        let dummy = TreeNode::new();
        let new_game = Game {
            state: game_state.clone(),
        };

        // Find the child corresponding to the new game state
        let chosen_edge = self
            .root
            .iter_mut_children(&self.root_game, None)
            .enumerate()
            .find(|(_, (child_move, _))| {
                let mut test_game = self.root_game.clone();
                test_game.make_move(*child_move);
                test_game.state == *game_state
            });

        if let Some((_, (_, chosen_child))) = chosen_edge {
            self.root = std::mem::replace(chosen_child, dummy);
        } else {
            self.root = dummy;
        }

        self.root_game = new_game;
        self.num_nodes
            .store(self.calculate_tree_size(), Ordering::Relaxed);
    }

    pub fn playout(&self) {
        let (path, game) = playout(&self.root, &self.root_game, &self.num_nodes);
        backprop(&self.root, &self.root_game, path, game);
    }

    pub fn take_action(&mut self) -> htmf::game::Action {
        if self.root_game.state.active_player() != Some(self.me) {
            panic!("{:?} was asked to move, but it is not their turn!", self.me);
        }
        playout(&self.root, &self.root_game, &self.num_nodes);
        let (best_move, _) = self
            .root
            .iter_mut_children(&self.root_game, Some(&self.num_nodes))
            .max_by(|(_, child1), (_, child2)| {
                let (child1_rewards, child1_visits) = child1.rewards_visits.get();
                let (child2_rewards, child2_visits) = child2.rewards_visits.get();
                (child1_rewards / child1_visits as f32)
                    .partial_cmp(&(child2_rewards / child2_visits as f32))
                    .unwrap_or(child1_visits.cmp(&child2_visits))
            })
            .unwrap();
        match *best_move {
            Move::Move((src, dst)) => htmf::game::Action::Move(src, dst),
            Move::Place(dst) => htmf::game::Action::Place(dst),
        }
    }

    pub fn tree_size(&self) -> usize {
        self.num_nodes.load(Ordering::Relaxed)
    }

    pub fn tree_size_bytes(&self) -> usize {
        self.root.size_in_bytes()
    }

    fn calculate_tree_size(&self) -> usize {
        let mut stack = vec![&self.root];
        let mut sum = 0;
        while let Some(node) = stack.pop() {
            sum += 1;
            if let Some(children) = node.children.get() {
                for (_, child) in children {
                    stack.push(child);
                }
            }
        }
        sum
    }
}

impl TreeNode {
    pub fn size_in_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.heap_size()
    }

    fn heap_size(&self) -> usize {
        let mut size = 0;
        if let Some(children) = self.children.get() {
            size += children.capacity() * std::mem::size_of::<(Move, TreeNode)>();
            for (_, child) in children {
                size += child.heap_size();
            }
        }
        size
    }
}

#[test]
fn test_run_full_game() {
    use htmf::board::Player;
    use htmf::game::GameState;

    let mut game = GameState::new_two_player::<StdRng>(&mut SeedableRng::seed_from_u64(0));
    let mut bots = (0..=1)
        .map(|i| MCTSBot::new(game.clone(), Player { id: i }))
        .collect::<Vec<MCTSBot>>();

    while let Some(p) = game.active_player() {
        bots[p.id].playout();
        let action = bots[p.id].take_action();
        game.apply_action(&action).unwrap();
        for bot in &mut bots {
            bot.update(&game);
        }
    }
}

#[test]
fn test_tree_size_optimization() {
    use htmf::board::Player;
    use htmf::game::GameState;

    let game = GameState::new_two_player::<StdRng>(&mut SeedableRng::seed_from_u64(0));
    let mut bot = MCTSBot::new(game.clone(), Player { id: 0 });

    assert_eq!(bot.tree_size(), 1);

    bot.playout();
    // Playout expands at least one node (unless game over, which it isn't)
    assert!(bot.tree_size() > 1);

    // Verify tree size matches manual calculation
    assert_eq!(bot.tree_size(), bot.calculate_tree_size());

    // Run more playouts
    for _ in 0..10 {
        bot.playout();
    }
    assert_eq!(bot.tree_size(), bot.calculate_tree_size());

    // Update bot (make a move)
    let action = bot.take_action();
    let mut new_game = game.clone();
    new_game.apply_action(&action).unwrap();
    bot.update(&new_game);

    // After update, tree size should be recalculated and correct
    assert_eq!(bot.tree_size(), bot.calculate_tree_size());
}

#[test]
fn test_memory_usage() {
    use htmf::board::Player;
    use htmf::game::GameState;

    let game = GameState::new_two_player::<StdRng>(&mut SeedableRng::seed_from_u64(0));
    let bot = MCTSBot::new(game.clone(), Player { id: 0 });

    println!("Initial size: {} bytes", bot.tree_size_bytes());

    for _ in 0..100 {
        bot.playout();
    }

    println!("Size after 100 playouts: {} bytes", bot.tree_size_bytes());
    println!("Tree size (nodes): {}", bot.tree_size());
    println!(
        "Average bytes per node: {}",
        bot.tree_size_bytes() as f64 / bot.tree_size() as f64
    );
    assert!(bot.tree_size_bytes() as f64 / bot.tree_size() as f64 <= 48.0);
}

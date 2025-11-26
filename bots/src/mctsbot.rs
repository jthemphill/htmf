use crate::neuralnet::NeuralNet;
use htmf::board::Board;
use htmf::hex::Cube;
use rand::prelude::*;
use std::cell::OnceCell;
use std::sync::Arc;

const NUM_PLAYERS: usize = 2;

// Compressed movement policy constants
const NUM_PENGUINS: usize = 4;
const NUM_DIRECTIONS: usize = 6;
const MAX_DISTANCE: usize = 7;
#[allow(dead_code)]
const MOVEMENT_POLICY_SIZE: usize = NUM_PENGUINS * NUM_DIRECTIONS * MAX_DISTANCE; // 168

/// Exploration constant for PUCT formula (AlphaZero uses ~1.0-2.0)
const C_PUCT: f32 = 1.0;

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
    pub rewards_visits: RewardsVisits,
    pub children: OnceCell<Vec<(Move, TreeNode)>>,
    /// Prior probability from policy network (only used with neural network)
    pub prior: f32,
}

impl TreeNode {
    pub fn new() -> Self {
        Self {
            rewards_visits: Default::default(),
            children: Default::default(),
            prior: 1.0, // Uniform prior when no neural network
        }
    }

    pub fn with_prior(prior: f32) -> Self {
        Self {
            rewards_visits: Default::default(),
            children: Default::default(),
            prior,
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

    pub fn get_child_mut(&mut self, game_move: Move) -> &mut TreeNode {
        &mut self
            .children
            .get_mut()
            .unwrap()
            .iter_mut()
            .find(|(child_move, _)| *child_move == game_move)
            .unwrap()
            .1
    }

    pub fn iter_children<'a>(
        &'a self,
        game: &Game,
        node_count: Option<&mut usize>,
    ) -> impl Iterator<Item = &'a (Move, TreeNode)> {
        self.children
            .get_or_init(|| {
                let children: Vec<_> = game
                    .available_moves()
                    .map(|child_move| (child_move, TreeNode::new()))
                    .collect();

                if let Some(node_count) = node_count {
                    *node_count += children.len();
                }

                children
            })
            .iter()
    }

    pub fn iter_mut_children<'a>(
        &'a mut self,
        game: &Game,
        node_count: Option<&mut usize>,
    ) -> impl Iterator<Item = &'a mut (Move, TreeNode)> {
        self.children.get_or_init(|| {
            let children: Vec<_> = game
                .available_moves()
                .map(|child_move| (child_move, TreeNode::new()))
                .collect();

            if let Some(node_count) = node_count {
                *node_count += children.len();
            }

            children
        });
        self.children.get_mut().unwrap().iter_mut()
    }

    /// Expand children using policy network priors
    pub fn expand_with_priors(
        &mut self,
        game: &Game,
        policy_logits: &[f32],
        current_player: usize,
        node_count: &mut usize,
    ) {
        if self.children.get().is_some() {
            return; // Already expanded
        }

        let is_drafting = !game.state.finished_drafting();
        let moves: Vec<Move> = game.available_moves().collect();

        // Get sorted penguin list for consistent indexing (only needed for movement)
        let mut penguins: Vec<u8> = game.state.board.penguins[current_player].into_iter().collect();
        penguins.sort();

        // Convert logits to probabilities with softmax over legal moves only
        let mut max_logit = f32::NEG_INFINITY;
        for m in &moves {
            let idx = move_to_policy_index(m, is_drafting, &penguins);
            max_logit = max_logit.max(policy_logits[idx]);
        }

        let mut sum_exp = 0.0f32;
        let mut priors: Vec<f32> = Vec::with_capacity(moves.len());
        for m in &moves {
            let idx = move_to_policy_index(m, is_drafting, &penguins);
            let exp_val = (policy_logits[idx] - max_logit).exp();
            priors.push(exp_val);
            sum_exp += exp_val;
        }

        // Normalize
        for p in &mut priors {
            *p /= sum_exp;
        }

        let children: Vec<_> = moves
            .into_iter()
            .zip(priors)
            .map(|(child_move, prior)| (child_move, TreeNode::with_prior(prior)))
            .collect();

        *node_count += children.len();
        let _ = self.children.set(children);
    }
}

/// Convert a move from (src, dst) to (direction, distance)
/// Direction is 0-5 based on Cube::neighbors() order
/// Distance is 1-7 (number of cells traveled)
fn move_to_direction_distance(src: u8, dst: u8) -> Option<(usize, usize)> {
    let src_hex = Board::index_to_evenr(src);
    let dst_hex = Board::index_to_evenr(dst);
    let src_cube = Cube::from_evenr(&src_hex);
    let dst_cube = Cube::from_evenr(&dst_hex);

    // Calculate the delta in cube coordinates
    let dx = dst_cube.x - src_cube.x;
    let dy = dst_cube.y - src_cube.y;
    let dz = dst_cube.z - src_cube.z;

    // Determine direction based on which axis is constant (the other two change)
    // Direction 0: (+x, -y, 0z) East
    // Direction 1: (+x, 0y, -z) Northeast
    // Direction 2: (0x, +y, -z) Northwest
    // Direction 3: (-x, +y, 0z) West
    // Direction 4: (-x, 0y, +z) Southwest
    // Direction 5: (0x, -y, +z) Southeast

    let direction = if dz == 0 {
        // z constant: East (0) or West (3)
        if dx > 0 { 0 } else { 3 }
    } else if dy == 0 {
        // y constant: Northeast (1) or Southwest (4)
        if dx > 0 { 1 } else { 4 }
    } else if dx == 0 {
        // x constant: Northwest (2) or Southeast (5)
        if dy > 0 { 2 } else { 5 }
    } else {
        // Not a valid hex line move
        return None;
    };

    // Distance is the absolute delta on any non-zero axis
    let distance = dx.abs().max(dy.abs()).max(dz.abs()) as usize;

    if distance == 0 || distance > MAX_DISTANCE {
        return None;
    }

    Some((direction, distance))
}

/// Convert a move to its index in the policy output
/// For movement phase, this uses the compressed format: penguin_idx * 42 + direction * 7 + (distance - 1)
fn move_to_policy_index(m: &Move, is_drafting: bool, penguins: &[u8]) -> usize {
    match m {
        Move::Place(dst) => {
            debug_assert!(is_drafting);
            *dst as usize
        }
        Move::Move((src, dst)) => {
            debug_assert!(!is_drafting);
            // Find penguin index
            let penguin_idx = penguins.iter().position(|&p| p == *src).unwrap_or(0);
            // Get direction and distance
            if let Some((direction, distance)) = move_to_direction_distance(*src, *dst) {
                penguin_idx * (NUM_DIRECTIONS * MAX_DISTANCE)
                    + direction * MAX_DISTANCE
                    + (distance - 1)
            } else {
                // Fallback - should not happen with valid moves
                0
            }
        }
    }
}

#[derive(Default, Debug)]
pub struct RewardsVisits {
    rewards: f32,
    visits: u32,
}

impl RewardsVisits {
    pub fn get(&self) -> (f32, u32) {
        (self.rewards, self.visits)
    }

    pub fn get_and_increment_visits(&mut self) -> (f32, u32) {
        let result = (self.rewards, self.visits);
        self.visits += 1;
        result
    }

    pub fn add_reward(&mut self, reward: f32) {
        self.rewards += reward;
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
    game: &Game,
    rng: &'_ mut R,
    node_count: &mut usize,
) -> (Move, &'tree mut TreeNode) {
    let (_, total_visits) = node.rewards_visits.get_and_increment_visits();

    let mut chosen_idx = 0;
    let mut num_optimal: f64 = 0.0;
    let mut best_so_far: f32 = std::f32::NEG_INFINITY;
    for (idx, (_, child)) in node.iter_children(game, Some(node_count)).enumerate() {
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
            chosen_idx = idx;
            num_optimal = 1.0;
            best_so_far = score;
        } else if (score - best_so_far).abs() < std::f32::EPSILON {
            num_optimal += 1.0;
            if rng.random_bool(1.0 / num_optimal) {
                chosen_idx = idx;
            }
        }
    }
    let children = node.children.get_mut().unwrap();
    let (child_move, child) = &mut children[chosen_idx];
    (*child_move, child)
}

/// Choose child using PUCT formula (for neural network guided search)
fn choose_child_puct<'tree, R: Rng + ?Sized>(
    node: &'tree mut TreeNode,
    rng: &'_ mut R,
) -> (Move, &'tree mut TreeNode) {
    let (_, total_visits) = node.rewards_visits.get_and_increment_visits();
    let sqrt_total = (total_visits as f32).sqrt();

    let children = node.children.get_mut().expect("Node must be expanded before PUCT selection");

    let mut chosen_idx = 0;
    let mut num_optimal: f64 = 0.0;
    let mut best_so_far: f32 = f32::NEG_INFINITY;

    for (idx, (_, child)) in children.iter().enumerate() {
        let (child_rewards, child_visits) = child.rewards_visits.get();

        // PUCT formula: Q(s,a) + c_puct * P(s,a) * sqrt(N(s)) / (1 + N(s,a))
        let q_value = if child_visits == 0 {
            0.5 // Optimistic initial value
        } else {
            child_rewards / child_visits as f32
        };

        let exploration = C_PUCT * child.prior * sqrt_total / (1.0 + child_visits as f32);
        let score = q_value + exploration;

        if score > best_so_far {
            chosen_idx = idx;
            num_optimal = 1.0;
            best_so_far = score;
        } else if (score - best_so_far).abs() < f32::EPSILON {
            num_optimal += 1.0;
            if rng.random_bool(1.0 / num_optimal) {
                chosen_idx = idx;
            }
        }
    }

    let (child_move, child) = &mut children[chosen_idx];
    (*child_move, child)
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

fn playout(root: &mut TreeNode, root_game: &Game, node_count: &mut usize) -> (Vec<Move>, Game) {
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

/// Neural network guided playout - returns the path, value estimate, and the player whose perspective the value is from
fn playout_nn(
    root: &mut TreeNode,
    root_game: &Game,
    nn: &NeuralNet,
    node_count: &mut usize,
) -> (Vec<Move>, f32, usize) {
    let rng = &mut rand::rng();
    let mut path = vec![];
    let mut expand_node = root;
    let mut game = root_game.clone();

    // Traverse to a leaf node using PUCT
    while !expand_node.is_leaf() {
        let (child_move, child_node) = choose_child_puct(expand_node, rng);
        game.make_move(child_move);
        expand_node = child_node;
        path.push(child_move);
    }

    // At a leaf node - evaluate with neural network and expand
    if game.state.game_over() {
        // Terminal node - use actual game result
        // Need to figure out who the last player to move was
        let last_player = if !path.is_empty() {
            let mut temp_game = root_game.clone();
            let mut last_p = temp_game.current_player().id;
            for (i, m) in path.iter().enumerate() {
                if i < path.len() - 1 {
                    last_p = temp_game.current_player().id;
                    temp_game.make_move(*m);
                }
            }
            // The player who made the last move
            let mut temp_game2 = root_game.clone();
            for (i, m) in path.iter().enumerate() {
                if i == path.len() - 1 {
                    return (path, get_reward(&game.state, temp_game2.current_player().id), temp_game2.current_player().id);
                }
                temp_game2.make_move(*m);
            }
            last_p
        } else {
            root_game.current_player().id
        };
        (path, get_reward(&game.state, last_player), last_player)
    } else {
        // Non-terminal leaf - use neural network
        let current_player = game.current_player().id;
        let output = nn.predict(&game.state, current_player).expect("NN prediction failed");

        // Expand with policy priors
        expand_node.expand_with_priors(&game, &output.policy_logits, current_player, node_count);

        // Return value from current player's perspective
        (path, output.value, current_player)
    }
}

fn backprop(root: &mut TreeNode, root_game: &Game, path: Vec<Move>, game: Game) {
    let rewards: [f32; NUM_PLAYERS] = [get_reward(&game.state, 0), get_reward(&game.state, 1)];
    let mut backprop_node = root;
    let mut current_game = root_game.clone();

    for backprop_move in path {
        if backprop_node.is_leaf() {
            break;
        }
        if let Some(p) = current_game.state.active_player() {
            backprop_node = backprop_node.get_child_mut(backprop_move);
            backprop_node.rewards_visits.add_reward(rewards[p.id]);
            current_game.make_move(backprop_move);
        } else {
            break;
        }
    }
}

/// Backpropagate a value estimate from neural network
fn backprop_value(root: &mut TreeNode, root_game: &Game, path: Vec<Move>, leaf_value: f32, leaf_player: usize) {
    let mut backprop_node = root;
    let mut current_game = root_game.clone();

    for backprop_move in path {
        if backprop_node.is_leaf() {
            break;
        }
        if let Some(p) = current_game.state.active_player() {
            backprop_node = backprop_node.get_child_mut(backprop_move);
            // leaf_value is from the perspective of leaf_player
            // We flip it for the opponent
            let value_for_p = if p.id == leaf_player {
                leaf_value
            } else {
                1.0 - leaf_value
            };
            backprop_node.rewards_visits.add_reward(value_for_p);
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
    pub num_nodes: usize,
    /// Optional neural network for guided search
    nn: Option<Arc<NeuralNet>>,
}

impl MCTSBot {
    pub fn new(game: htmf::game::GameState, me: htmf::board::Player) -> Self {
        MCTSBot {
            root: TreeNode::new(),
            root_game: Game { state: game },
            me,
            num_nodes: 1,
            nn: None,
        }
    }

    /// Create an MCTS bot guided by a neural network
    pub fn with_neural_net(
        game: htmf::game::GameState,
        me: htmf::board::Player,
        nn: Arc<NeuralNet>,
    ) -> Self {
        let mut bot = MCTSBot {
            root: TreeNode::new(),
            root_game: Game { state: game.clone() },
            me,
            num_nodes: 1,
            nn: Some(nn.clone()),
        };

        // Initialize root with neural network priors
        if let Some(p) = game.active_player() {
            let output = nn.predict(&game, p.id).expect("NN prediction failed");
            bot.root.expand_with_priors(
                &bot.root_game,
                &output.policy_logits,
                p.id,
                &mut bot.num_nodes,
            );
        }

        bot
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
            // New state not found in tree - start fresh
            self.root = dummy;

            // If using neural network, initialize with priors
            if let (Some(nn), Some(p)) = (&self.nn, game_state.active_player()) {
                let output = nn.predict(game_state, p.id).expect("NN prediction failed");
                self.root.expand_with_priors(&new_game, &output.policy_logits, p.id, &mut self.num_nodes);
            }
        }

        self.root_game = new_game;
        self.num_nodes = self.calculate_tree_size();
    }

    pub fn playout(&mut self) {
        if let Some(nn) = &self.nn {
            // Neural network guided search
            let (path, value, leaf_player) = playout_nn(&mut self.root, &self.root_game, nn, &mut self.num_nodes);
            backprop_value(&mut self.root, &self.root_game, path, value, leaf_player);
        } else {
            // Traditional MCTS with random rollouts
            let (path, game) = playout(&mut self.root, &self.root_game, &mut self.num_nodes);
            backprop(&mut self.root, &self.root_game, path, game);
        }
    }

    pub fn take_action(&mut self) -> htmf::game::Action {
        self.take_action_with_temperature(0.0)
    }

    /// Take action with temperature-based sampling
    /// temperature = 0.0: always pick best move (greedy)
    /// temperature = 1.0: sample proportional to visit counts
    /// temperature > 1.0: more exploration
    pub fn take_action_with_temperature(&mut self, temperature: f32) -> htmf::game::Action {
        if self.root_game.state.active_player() != Some(self.me) {
            panic!("{:?} was asked to move, but it is not their turn!", self.me);
        }
        self.playout();

        // Expand if needed
        if self.root.children.get().is_none() {
            let _: Vec<_> = self.root.iter_mut_children(&self.root_game, Some(&mut self.num_nodes)).collect();
        }

        let children = self.root.children.get().unwrap();

        let best_move = if temperature < 0.01 {
            // Greedy: pick move with highest visit count
            children
                .iter()
                .max_by_key(|(_, child)| {
                    let (_, visits) = child.rewards_visits.get();
                    visits
                })
                .map(|(m, _)| *m)
                .unwrap()
        } else {
            // Temperature-based sampling
            let rng = &mut rand::rng();

            // Get visit counts
            let visits: Vec<f32> = children
                .iter()
                .map(|(_, child)| {
                    let (_, v) = child.rewards_visits.get();
                    v as f32
                })
                .collect();

            // Apply temperature: p_i = N_i^(1/T) / sum(N_j^(1/T))
            let inv_temp = 1.0 / temperature;
            let powered: Vec<f32> = visits.iter().map(|v| v.powf(inv_temp)).collect();
            let sum: f32 = powered.iter().sum();

            if sum < 1e-10 {
                // Fallback to uniform if no visits
                children.choose(rng).map(|(m, _)| *m).unwrap()
            } else {
                let probs: Vec<f32> = powered.iter().map(|p| p / sum).collect();

                // Sample from distribution
                let mut cumsum = 0.0;
                let r: f32 = rng.random();
                let mut chosen_idx = 0;
                for (i, p) in probs.iter().enumerate() {
                    cumsum += p;
                    if r < cumsum {
                        chosen_idx = i;
                        break;
                    }
                }
                children[chosen_idx].0
            }
        };

        match best_move {
            Move::Move((src, dst)) => htmf::game::Action::Move(src, dst),
            Move::Place(dst) => htmf::game::Action::Place(dst),
        }
    }

    pub fn tree_size(&self) -> usize {
        self.num_nodes
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
    let mut bot = MCTSBot::new(game.clone(), Player { id: 0 });

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
    assert!(bot.tree_size_bytes() as f64 / bot.tree_size() as f64 <= 56.0);
}

#[test]
#[ignore] // Requires model files to be present
fn test_neural_network_guided_game() {
    use htmf::board::Player;
    use htmf::game::GameState;

    // Load neural network
    let nn = Arc::new(
        NeuralNet::load(
            "../training/artifacts/model_drafting.onnx",
            "../training/artifacts/model_movement.onnx",
        )
        .expect("Failed to load neural network"),
    );

    let mut game = GameState::new_two_player::<StdRng>(&mut SeedableRng::seed_from_u64(42));
    let mut bots = (0..=1)
        .map(|i| MCTSBot::with_neural_net(game.clone(), Player { id: i }, nn.clone()))
        .collect::<Vec<MCTSBot>>();

    let mut move_count = 0;
    while let Some(p) = game.active_player() {
        // Run a few playouts
        for _ in 0..10 {
            bots[p.id].playout();
        }
        let action = bots[p.id].take_action();
        game.apply_action(&action).unwrap();
        for bot in &mut bots {
            bot.update(&game);
        }
        move_count += 1;
    }

    println!("Game completed in {} moves", move_count);
    println!("Final scores: {:?}", game.get_scores());
    assert!(move_count > 8); // At least drafting phase completed
}

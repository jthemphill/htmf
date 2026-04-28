use crate::neuralnet::NeuralNet;
use crate::policy::{move_to_policy_index, policy_size};
use rand::prelude::*;
use std::cell::OnceCell;
use std::fmt;
use std::sync::Arc;

const NUM_PLAYERS: usize = 2;

/// Exploration constant for PUCT formula (AlphaZero uses ~1.0-2.0)
const C_PUCT: f32 = 1.0;
/// Keep the production uniform prior as an anchor while the learned policy is weak.
const DEFAULT_MODEL_PRIOR_WEIGHT: f32 = 0.05;

fn model_prior_weight() -> f32 {
    std::env::var("HTMF_MODEL_PRIOR_WEIGHT")
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
        .filter(|value| (0.0..=1.0).contains(value))
        .unwrap_or(DEFAULT_MODEL_PRIOR_WEIGHT)
}

#[derive(Debug)]
pub struct PriorError {
    message: String,
}

impl PriorError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for PriorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for PriorError {}

pub trait PriorProvider: Send + Sync {
    fn policy_logits(
        &self,
        game: &htmf::game::GameState,
        current_player: usize,
    ) -> Result<Vec<f32>, PriorError>;
}

#[derive(Debug, Default)]
pub struct UniformPriorProvider;

impl PriorProvider for UniformPriorProvider {
    fn policy_logits(
        &self,
        game: &htmf::game::GameState,
        _current_player: usize,
    ) -> Result<Vec<f32>, PriorError> {
        Ok(vec![0.0; policy_size(!game.finished_drafting())])
    }
}

impl PriorProvider for NeuralNet {
    fn policy_logits(
        &self,
        game: &htmf::game::GameState,
        current_player: usize,
    ) -> Result<Vec<f32>, PriorError> {
        self.predict(game, current_player)
            .map(|output| output.policy_logits)
            .map_err(|err| PriorError::new(format!("ONNX prior inference failed: {err}")))
    }
}

/**
 * Games are connected to each other via Moves.
 */
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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
        _current_player: usize,
        node_count: &mut usize,
    ) {
        if self.children.get().is_some() {
            return; // Already expanded
        }

        let is_drafting = !game.state.finished_drafting();
        let moves: Vec<Move> = game.available_moves().collect();

        // Convert logits to probabilities with softmax over legal moves only
        let mut max_logit = f32::NEG_INFINITY;
        for m in &moves {
            let idx = move_to_policy_index(m, is_drafting);
            max_logit = max_logit.max(policy_logits.get(idx).copied().unwrap_or(0.0));
        }

        let mut sum_exp = 0.0f32;
        let mut priors: Vec<f32> = Vec::with_capacity(moves.len());
        for m in &moves {
            let idx = move_to_policy_index(m, is_drafting);
            let exp_val = (policy_logits.get(idx).copied().unwrap_or(0.0) - max_logit).exp();
            priors.push(exp_val);
            sum_exp += exp_val;
        }

        // Normalize and blend with uniform so weak models cannot completely
        // dominate the production baseline search.
        let model_prior_weight = model_prior_weight();
        let uniform_prior = 1.0 / priors.len() as f32;
        for p in &mut priors {
            *p = model_prior_weight * (*p / sum_exp)
                + (1.0 - model_prior_weight) * uniform_prior;
        }

        let children: Vec<_> = moves
            .into_iter()
            .zip(priors)
            .map(|(child_move, prior)| (child_move, TreeNode::with_prior(prior)))
            .collect();

        *node_count += children.len();
        let _ = self.children.set(children);
    }

    /// Expand children with uniform priors (1/n for each of n children)
    pub fn expand_with_uniform_priors(&mut self, game: &Game, node_count: &mut usize) {
        if self.children.get().is_some() {
            return; // Already expanded
        }

        let moves: Vec<Move> = game.available_moves().collect();
        let uniform_prior = 1.0 / moves.len() as f32;

        let children: Vec<_> = moves
            .into_iter()
            .map(|child_move| (child_move, TreeNode::with_prior(uniform_prior)))
            .collect();

        *node_count += children.len();
        let _ = self.children.set(children);
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

/// Choose child using PUCT formula
fn choose_child_puct<'tree, R: Rng + ?Sized>(
    node: &'tree mut TreeNode,
    rng: &'_ mut R,
) -> (Move, &'tree mut TreeNode) {
    let (_, total_visits) = node.rewards_visits.get_and_increment_visits();
    let sqrt_total = (total_visits as f32).sqrt();

    let children = node
        .children
        .get_mut()
        .expect("Node must be expanded before PUCT selection");

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

/// PUCT-based playout with random rollout evaluation.
/// Uses neural network policy priors if available, otherwise uniform priors.
fn playout_puct(
    root: &mut TreeNode,
    root_game: &Game,
    prior_provider: &Arc<dyn PriorProvider>,
    node_count: &mut usize,
) -> (Vec<Move>, Game) {
    let rng = &mut rand::rng();
    let mut path = vec![];
    let mut expand_node = root;
    let mut game = root_game.clone();

    // Traverse using PUCT
    while !expand_node.is_leaf() {
        let (child_move, child_node) = choose_child_puct(expand_node, rng);
        game.make_move(child_move);
        expand_node = child_node;
        path.push(child_move);
    }

    // At a leaf - expand with priors (from NN if available, otherwise uniform)
    if !game.state.game_over() {
        let current_player = game.current_player().id;
        match prior_provider.policy_logits(&game.state, current_player) {
            Ok(policy_logits) => {
                expand_node.expand_with_priors(&game, &policy_logits, current_player, node_count);
            }
            Err(_) => expand_node.expand_with_uniform_priors(&game, node_count),
        }

        // Select one child to expand into (using PUCT)
        let (child_move, _child_node) = choose_child_puct(expand_node, rng);
        game.make_move(child_move);
        path.push(child_move);
    }

    // Random rollout to end of game
    while let Some(game_move) = game.available_moves().choose(rng) {
        path.push(game_move);
        game.make_move(game_move);
    }

    (path, game)
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

#[derive(Clone, Copy, Debug)]
pub struct UpdateStats {
    pub old_size: usize,
    pub old_capacity: usize,
    pub new_size: usize,
    pub new_capacity: usize,
}

/// Mode of operation for MCTS
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MCTSMode {
    /// Production baseline: PUCT with uniform priors and random rollouts.
    Pure,
    /// PUCT selection with random rollouts.
    /// Uses neural network policy priors when a model is provided.
    NeuralNet,
}

pub struct MCTSBot {
    pub root: TreeNode,
    pub root_game: Game,
    pub me: htmf::board::Player,
    pub num_nodes: usize,
    /// Source of policy priors used by PUCT.
    prior_provider: Arc<dyn PriorProvider>,
}

impl MCTSBot {
    pub fn new(game: htmf::game::GameState, me: htmf::board::Player) -> Self {
        Self::with_prior_provider(game, me, Arc::new(UniformPriorProvider), MCTSMode::Pure)
    }

    /// Create an MCTS bot that uses PUCT selection with random rollouts.
    ///
    /// If a neural network is provided, it will be used for policy priors
    /// (guiding which moves to explore first). Otherwise, uniform priors are used.
    ///
    /// This mode uses random rollouts for leaf evaluation (not NN value prediction),
    /// matching the uniform-prior production baseline. Training can then
    /// incrementally improve the policy priors.
    pub fn with_neural_net(
        game: htmf::game::GameState,
        me: htmf::board::Player,
        nn: Option<Arc<NeuralNet>>,
    ) -> Self {
        let prior_provider: Arc<dyn PriorProvider> = nn
            .map(|nn| nn as Arc<dyn PriorProvider>)
            .unwrap_or_else(|| Arc::new(UniformPriorProvider));
        Self::with_prior_provider(game, me, prior_provider, MCTSMode::NeuralNet)
    }

    pub fn with_prior_provider(
        game: htmf::game::GameState,
        me: htmf::board::Player,
        prior_provider: Arc<dyn PriorProvider>,
        _mode: MCTSMode,
    ) -> Self {
        let mut bot = MCTSBot {
            root: TreeNode::new(),
            root_game: Game {
                state: game.clone(),
            },
            me,
            num_nodes: 1,
            prior_provider,
        };

        // Initialize root with priors. The uniform provider is the production baseline.
        if let Some(p) = game.active_player() {
            bot.expand_node_with_priors(&bot.root_game.clone(), p.id);
        }

        bot
    }

    /// Expand a node with priors (from NN if available, otherwise uniform)
    fn expand_node_with_priors(&mut self, game: &Game, current_player: usize) {
        match self
            .prior_provider
            .policy_logits(&game.state, current_player)
        {
            Ok(policy_logits) => {
                self.root.expand_with_priors(
                    game,
                    &policy_logits,
                    current_player,
                    &mut self.num_nodes,
                );
            }
            Err(_) => {
                self.root
                    .expand_with_uniform_priors(game, &mut self.num_nodes);
            }
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
            // New state not found in tree - start fresh
            self.root = dummy;

            if let Some(p) = game_state.active_player() {
                self.expand_node_with_priors(&new_game, p.id);
            }
        }

        self.root_game = new_game;
        self.num_nodes = self.calculate_tree_size();
    }

    pub fn playout(&mut self) {
        let (path, game) = playout_puct(
            &mut self.root,
            &self.root_game,
            &self.prior_provider,
            &mut self.num_nodes,
        );
        backprop(&mut self.root, &self.root_game, path, game);
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
            let _: Vec<_> = self
                .root
                .iter_mut_children(&self.root_game, Some(&mut self.num_nodes))
                .collect();
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

    pub fn update_root_priors_from_logits(&mut self, policy_logits: &[f32]) {
        if self.root_game.state.active_player().is_none() {
            return;
        }
        let Some(children) = self.root.children.get_mut() else {
            return;
        };

        let is_drafting = !self.root_game.state.finished_drafting();
        let mut max_logit = f32::NEG_INFINITY;
        for (m, _) in children.iter() {
            let idx = move_to_policy_index(m, is_drafting);
            max_logit = max_logit.max(policy_logits.get(idx).copied().unwrap_or(0.0));
        }

        let mut sum_exp = 0.0f32;
        let mut priors = Vec::with_capacity(children.len());
        for (m, _) in children.iter() {
            let idx = move_to_policy_index(m, is_drafting);
            let exp_val = (policy_logits.get(idx).copied().unwrap_or(0.0) - max_logit).exp();
            priors.push(exp_val);
            sum_exp += exp_val;
        }

        if sum_exp <= 0.0 || !sum_exp.is_finite() {
            return;
        }

        let model_prior_weight = model_prior_weight();
        let uniform_prior = 1.0 / priors.len() as f32;
        for ((_, child), prior) in children.iter_mut().zip(priors) {
            child.prior = model_prior_weight * (prior / sum_exp)
                + (1.0 - model_prior_weight) * uniform_prior;
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

    // Root is now pre-expanded with uniform priors (PUCT requires this)
    let initial_size = bot.tree_size();
    assert!(initial_size > 1, "Root should be pre-expanded");

    bot.playout();
    // Playout expands at least one more node
    assert!(bot.tree_size() > initial_size);

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
fn test_uniform_prior_paths_match_at_root() {
    use htmf::board::Player;
    use htmf::game::GameState;

    let game = GameState::new_two_player::<StdRng>(&mut SeedableRng::seed_from_u64(7));
    let baseline = MCTSBot::new(game.clone(), Player { id: 0 });
    let explicit_uniform = MCTSBot::with_neural_net(game, Player { id: 0 }, None);

    let baseline_children = baseline.root.children.get().unwrap();
    let explicit_children = explicit_uniform.root.children.get().unwrap();

    assert_eq!(baseline_children.len(), explicit_children.len());
    for ((baseline_move, baseline_child), (uniform_move, uniform_child)) in
        baseline_children.iter().zip(explicit_children)
    {
        assert_eq!(baseline_move, uniform_move);
        assert_eq!(baseline_child.prior, uniform_child.prior);
        assert_eq!(
            baseline_child.rewards_visits.get(),
            uniform_child.rewards_visits.get()
        );
    }
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
        NeuralNet::load("../training/artifacts/model.onnx").expect("Failed to load neural network"),
    );

    let mut game = GameState::new_two_player::<StdRng>(&mut SeedableRng::seed_from_u64(42));
    let mut bots = (0..=1)
        .map(|i| MCTSBot::with_neural_net(game.clone(), Player { id: i }, Some(nn.clone())))
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

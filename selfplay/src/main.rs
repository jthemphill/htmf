extern crate rand;
extern crate rayon;
extern crate serde;
extern crate serde_json;

extern crate htmf;
extern crate htmf_bots;

use std::sync::{mpsc, Arc};

use rayon::prelude::*;
use serde::Serialize;

use htmf::board::*;
use htmf::game::*;
use htmf::hex::Cube;
use htmf::NUM_CELLS;
use htmf_bots::mctsbot::*;
use htmf_bots::NeuralNet;

// Compressed movement policy: 4 penguins × 6 directions × 7 max distances = 168 values
const NUM_PENGUINS: usize = 4;
const NUM_DIRECTIONS: usize = 6;
const MAX_DISTANCE: usize = 7;
pub const MOVEMENT_POLICY_SIZE: usize = NUM_PENGUINS * NUM_DIRECTIONS * MAX_DISTANCE; // 168

const NUM_PLAYERS: usize = 2;

/// A training sample containing the game state, MCTS policy, and eventual outcome
#[derive(Debug, Clone, Serialize)]
pub struct TrainingSample {
    /// Board features as a flat array
    /// Layout: 8 channels x 60 cells = 480 values
    /// Channels:
    ///   0: 1-fish cells
    ///   1: 2-fish cells
    ///   2: 3-fish cells
    ///   3: current player's penguins
    ///   4: opponent's penguins
    ///   5: current player's claimed cells
    ///   6: opponent's claimed cells
    ///   7: is drafting phase (all 1s or all 0s)
    pub features: Vec<f32>,
    /// MCTS visit distribution over all possible moves (policy target)
    /// For placement: 60 values (one per cell)
    /// For movement: 168 values (4 penguins × 6 directions × 7 distances)
    ///   Index = penguin_idx * 42 + direction * 7 + (distance - 1)
    pub policy: Vec<f32>,
    /// Game outcome from current player's perspective: 1.0 = win, 0.5 = draw, 0.0 = loss
    pub value: f32,
    /// Current player (0 or 1)
    pub player: usize,
    /// Whether this is drafting phase
    pub is_drafting: bool,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Check for --nn flag
    let use_nn = args.iter().any(|a| a == "--nn");
    let numeric_args: Vec<&String> = args.iter().skip(1).filter(|a| !a.starts_with("--")).collect();

    let ntrials: usize = numeric_args
        .first()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);
    let nplayouts: usize = numeric_args
        .get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(if use_nn { 200 } else { 14_000 });

    // Load neural network if requested
    let nn: Option<Arc<NeuralNet>> = if use_nn {
        eprintln!("Loading neural network...");
        match NeuralNet::load(
            "training/artifacts/model_drafting.onnx",
            "training/artifacts/model_movement.onnx",
        ) {
            Ok(model) => {
                eprintln!("Neural network loaded successfully");
                Some(Arc::new(model))
            }
            Err(e) => {
                eprintln!("Failed to load neural network: {:?}", e);
                eprintln!("Falling back to traditional MCTS");
                None
            }
        }
    } else {
        None
    };

    let mode = if nn.is_some() { "NN-guided MCTS" } else { "Traditional MCTS" };
    eprintln!(
        "Running {} games with {} playouts per move ({})",
        ntrials, nplayouts, mode
    );

    let (logger_tx, logger_rx) = mpsc::channel();
    let logger_handle = std::thread::spawn(move || {
        let mut total_samples = 0;
        loop {
            let res: Result<Vec<TrainingSample>, ()> = logger_rx.recv().unwrap();
            match res {
                Ok(samples) => {
                    for sample in &samples {
                        println!("{}", serde_json::to_string(sample).unwrap());
                    }
                    total_samples += samples.len();
                }
                Err(()) => break,
            }
        }
        total_samples
    });

    let results: Vec<(usize, usize)> = (0..ntrials)
        .into_par_iter()
        .map_with(nn.clone(), |nn, _| play_game(nplayouts, nn.clone()))
        .map_with(mpsc::Sender::clone(&logger_tx), |logger_tx, result| {
            let _ = logger_tx.send(Ok(result.samples));
            (result.winner, result.num_moves)
        })
        .collect();

    // Stop the logger
    logger_tx.send(Err(())).unwrap();
    let total_samples = logger_handle.join().unwrap();

    let wins: [usize; 2] = results.iter().fold([0, 0], |mut acc, (winner, _)| {
        acc[*winner] += 1;
        acc
    });
    let total_moves: usize = results.iter().map(|(_, moves)| moves).sum();

    eprintln!("Player 0 wins: {} / {}", wins[0], ntrials);
    eprintln!("Player 1 wins: {} / {}", wins[1], ntrials);
    eprintln!("Total moves: {}", total_moves);
    eprintln!("Total training samples: {}", total_samples);
}

struct GameResult {
    winner: usize,
    samples: Vec<TrainingSample>,
    num_moves: usize,
}

/// Extract board features for neural network input
fn extract_features(game: &GameState, current_player: usize) -> Vec<f32> {
    let opponent = 1 - current_player;
    let is_drafting = !game.finished_drafting();

    let mut features = vec![0.0f32; 8 * NUM_CELLS];

    // Channels 0-2: fish counts
    for cell in 0..NUM_CELLS as u8 {
        if game.board.fish[0].contains(cell) {
            features[0 * NUM_CELLS + cell as usize] = 1.0;
        }
        if game.board.fish[1].contains(cell) {
            features[1 * NUM_CELLS + cell as usize] = 1.0;
        }
        if game.board.fish[2].contains(cell) {
            features[2 * NUM_CELLS + cell as usize] = 1.0;
        }
    }

    // Channel 3: current player's penguins
    for cell in game.board.penguins[current_player].into_iter() {
        features[3 * NUM_CELLS + cell as usize] = 1.0;
    }

    // Channel 4: opponent's penguins
    for cell in game.board.penguins[opponent].into_iter() {
        features[4 * NUM_CELLS + cell as usize] = 1.0;
    }

    // Channel 5: current player's claimed cells
    for cell in game.board.claimed[current_player].into_iter() {
        features[5 * NUM_CELLS + cell as usize] = 1.0;
    }

    // Channel 6: opponent's claimed cells
    for cell in game.board.claimed[opponent].into_iter() {
        features[6 * NUM_CELLS + cell as usize] = 1.0;
    }

    // Channel 7: drafting phase indicator
    if is_drafting {
        for i in 0..NUM_CELLS {
            features[7 * NUM_CELLS + i] = 1.0;
        }
    }

    features
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

/// Extract MCTS policy from tree node visit counts
fn extract_policy(mcts: &MCTSBot, game: &GameState, current_player: usize) -> Vec<f32> {
    let is_drafting = !game.finished_drafting();

    if is_drafting {
        // Placement policy: 60 values
        let mut policy = vec![0.0f32; NUM_CELLS];
        let mut total_visits = 0u32;

        if let Some(children) = mcts.root.children.get() {
            for (mov, child) in children {
                if let Move::Place(dst) = mov {
                    let (_, visits) = child.rewards_visits.get();
                    policy[*dst as usize] = visits as f32;
                    total_visits += visits;
                }
            }
        }

        // Normalize to probability distribution
        if total_visits > 0 {
            for p in &mut policy {
                *p /= total_visits as f32;
            }
        }

        policy
    } else {
        // Compressed movement policy: 4 penguins × 6 directions × 7 distances = 168 values
        let mut policy = vec![0.0f32; MOVEMENT_POLICY_SIZE];
        let mut total_visits = 0u32;

        // Get current player's penguins in sorted order for consistent indexing
        let mut penguins: Vec<u8> = game.board.penguins[current_player].into_iter().collect();
        penguins.sort();

        if let Some(children) = mcts.root.children.get() {
            for (mov, child) in children {
                if let Move::Move((src, dst)) = mov {
                    // Find which penguin index this is
                    let penguin_idx = penguins.iter().position(|&p| p == *src);
                    if let Some(penguin_idx) = penguin_idx {
                        if let Some((direction, distance)) = move_to_direction_distance(*src, *dst) {
                            let (_, visits) = child.rewards_visits.get();
                            // Index = penguin_idx * 42 + direction * 7 + (distance - 1)
                            let idx = penguin_idx * (NUM_DIRECTIONS * MAX_DISTANCE)
                                    + direction * MAX_DISTANCE
                                    + (distance - 1);
                            policy[idx] = visits as f32;
                            total_visits += visits;
                        }
                    }
                }
            }
        }

        // Normalize to probability distribution
        if total_visits > 0 {
            for p in &mut policy {
                *p /= total_visits as f32;
            }
        }

        policy
    }
}

/// Temperature schedule for move selection
/// High temperature early = more exploration
/// Low temperature later = more exploitation
fn get_temperature(move_num: usize) -> f32 {
    if move_num < 15 {
        1.0 // High exploration for first 15 moves
    } else if move_num < 30 {
        0.5 // Medium exploration
    } else {
        0.1 // Low exploration (nearly greedy)
    }
}

fn play_game(nplayouts: usize, nn: Option<Arc<NeuralNet>>) -> GameResult {
    let mut game = GameState::new_two_player(&mut rand::rng());
    let mut bots: Vec<MCTSBot> = (0..NUM_PLAYERS)
        .map(|i| {
            if let Some(ref nn) = nn {
                MCTSBot::with_neural_net(game.clone(), Player { id: i }, nn.clone())
            } else {
                MCTSBot::new(game.clone(), Player { id: i })
            }
        })
        .collect();

    // Store intermediate samples (without final value)
    struct PendingSample {
        features: Vec<f32>,
        policy: Vec<f32>,
        player: usize,
        is_drafting: bool,
    }
    let mut pending_samples: Vec<PendingSample> = vec![];
    let mut num_moves = 0;

    while let Some(p) = game.active_player() {
        // Run MCTS playouts
        for _ in 0..nplayouts {
            bots[p.id].playout();
        }

        // Extract training data BEFORE taking the action
        let features = extract_features(&game, p.id);
        let policy = extract_policy(&bots[p.id], &game, p.id);
        let is_drafting = !game.finished_drafting();

        pending_samples.push(PendingSample {
            features,
            policy,
            player: p.id,
            is_drafting,
        });

        // Take the action with temperature-based exploration
        let temperature = get_temperature(num_moves);
        let action = bots[p.id].take_action_with_temperature(temperature);
        num_moves += 1;

        match action {
            Action::Move(src, dst) => {
                game.move_penguin(src, dst).expect("Illegal move");
            }
            Action::Place(dst) => {
                game.place_penguin(dst).expect("Illegal placement");
                if game.board.is_cut_cell(dst) {
                    game.board.prune();
                }
                game.board.reap();
            }
            _ => panic!("Unexpected action received"),
        };

        // Update all bots
        for bot in &mut bots {
            bot.update(&game);
        }
    }

    // Determine winner and compute values
    let scores = game.get_scores();
    let winner = if scores[0] > scores[1] {
        0
    } else if scores[1] > scores[0] {
        1
    } else {
        // Draw - pick 0 arbitrarily but values will be 0.5
        0
    };

    let values: [f32; 2] = if scores[0] == scores[1] {
        [0.5, 0.5]
    } else if scores[0] > scores[1] {
        [1.0, 0.0]
    } else {
        [0.0, 1.0]
    };

    // Convert pending samples to final training samples
    let samples: Vec<TrainingSample> = pending_samples
        .into_iter()
        .map(|s| TrainingSample {
            features: s.features,
            policy: s.policy,
            value: values[s.player],
            player: s.player,
            is_drafting: s.is_drafting,
        })
        .collect();

    GameResult {
        winner,
        samples,
        num_moves,
    }
}

#[test]
fn test_selfplay_generates_samples() {
    let nplayouts = 50;
    let result = play_game(nplayouts, None);

    // Should have samples for each move in the game
    assert!(
        !result.samples.is_empty(),
        "Should generate training samples"
    );
    assert!(result.num_moves > 0, "Should have played some moves");
    assert_eq!(
        result.samples.len(),
        result.num_moves,
        "One sample per move"
    );

    // Check first sample has correct feature dimensions
    let first = &result.samples[0];
    assert_eq!(
        first.features.len(),
        8 * NUM_CELLS,
        "Features should be 8 channels x 60 cells"
    );

    // First 8 moves are drafting
    for sample in result.samples.iter().take(8) {
        assert!(
            sample.is_drafting,
            "First 8 samples should be drafting phase"
        );
        assert_eq!(
            sample.policy.len(),
            NUM_CELLS,
            "Drafting policy should be 60 values"
        );
    }

    // After drafting, policy should be movement (168 values - compressed)
    if result.samples.len() > 8 {
        let movement_sample = &result.samples[8];
        assert!(
            !movement_sample.is_drafting,
            "Sample 9+ should be movement phase"
        );
        assert_eq!(
            movement_sample.policy.len(),
            MOVEMENT_POLICY_SIZE,
            "Movement policy should be 168 values (4 penguins × 6 directions × 7 distances)"
        );
    }

    // Values should be valid (0.0, 0.5, or 1.0)
    for sample in &result.samples {
        assert!(
            sample.value == 0.0 || sample.value == 0.5 || sample.value == 1.0,
            "Value should be 0, 0.5, or 1"
        );
    }
}

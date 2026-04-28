extern crate rand;
extern crate rayon;
extern crate serde;
extern crate serde_json;

extern crate htmf;
extern crate htmf_bots;

use std::sync::{mpsc, Arc};
use std::time::{SystemTime, UNIX_EPOCH};

use rayon::prelude::*;
use serde::Serialize;

use htmf::board::*;
use htmf::game::*;
use htmf::NUM_CELLS;
use htmf_bots::mctsbot::*;
use htmf_bots::policy::{move_to_policy_index, MOVEMENT_POLICY_SIZE, POLICY_VERSION};
use htmf_bots::NeuralNet;

const NUM_PLAYERS: usize = 2;

#[derive(Debug, Clone, Serialize)]
pub struct TeacherMetadata {
    pub teacher: String,
    pub teacher_playouts: usize,
    pub model_prior_weight: f32,
    pub run_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub teacher_model: Option<String>,
}

/// A training sample containing the game state, MCTS policy, and eventual outcome
#[derive(Debug, Clone, Serialize)]
pub struct TrainingSample {
    /// Policy encoding version. Version 2 uses absolute movement source-cell encoding.
    pub policy_version: u8,
    /// Teacher/search configuration that generated this policy target.
    pub teacher: String,
    pub teacher_playouts: usize,
    pub model_prior_weight: f32,
    pub run_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub teacher_model: Option<String>,
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
    /// For movement: 2520 values (60 source cells × 6 directions × 7 distances)
    ///   Index = source_cell * 42 + direction * 7 + (distance - 1)
    pub policy: Vec<f32>,
    /// Game outcome from current player's perspective: 1.0 = win, 0.5 = draw, 0.0 = loss
    pub value: f32,
    /// Current player (0 or 1)
    pub player: usize,
    /// Zero-based move number when this position was searched.
    pub turn: usize,
    /// Whether this is drafting phase
    pub is_drafting: bool,
    /// Ownership prediction target: which player owns each cell at game end
    /// Array of 60 values, each in [0, 1, 2]:
    ///   0 = player 0 claimed this cell
    ///   1 = player 1 claimed this cell
    ///   2 = neither player claimed this cell
    pub ownership: Vec<u8>,
    /// Score difference prediction target (from current player's perspective)
    /// Stored as bin index in range [0, 184] where:
    ///   bin_index = (player_score - opponent_score) - (-92) = score_diff + 92
    pub score_diff: u8,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Check for --nn flag
    let use_nn = args.iter().any(|a| a == "--nn");
    let numeric_args: Vec<&String> = args
        .iter()
        .skip(1)
        .filter(|a| !a.starts_with("--"))
        .collect();

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
        match NeuralNet::load("training/artifacts/model.onnx") {
            Ok(model) => {
                eprintln!("Neural network loaded successfully");
                Some(Arc::new(model))
            }
            Err(e) => {
                eprintln!("Failed to load neural network: {:?}", e);
                eprintln!("Falling back to uniform priors");
                None
            }
        }
    } else {
        None
    };

    let metadata = Arc::new(build_teacher_metadata(nplayouts, nn.is_some(), use_nn));

    let mode = if nn.is_some() {
        "NN-guided priors"
    } else {
        "uniform-prior baseline"
    };
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
        .map_with((nn.clone(), metadata.clone()), |state, _| {
            let (nn, metadata) = state;
            play_game(nplayouts, nn.clone(), metadata.as_ref())
        })
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

fn build_teacher_metadata(
    nplayouts: usize,
    nn_loaded: bool,
    nn_requested: bool,
) -> TeacherMetadata {
    let requested_teacher = std::env::var("HTMF_SELFPLAY_TEACHER").unwrap_or_else(|_| {
        if nn_requested {
            "nn_root".to_owned()
        } else {
            "uniform".to_owned()
        }
    });
    let teacher = if nn_loaded && requested_teacher == "nn_root" {
        "nn_root".to_owned()
    } else {
        "uniform".to_owned()
    };
    let model_prior_weight = if teacher == "nn_root" {
        std::env::var("HTMF_MODEL_PRIOR_WEIGHT")
            .ok()
            .and_then(|value| value.parse::<f32>().ok())
            .unwrap_or(0.05)
    } else {
        0.0
    };
    let run_id = std::env::var("HTMF_SELFPLAY_RUN_ID").unwrap_or_else(|_| {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or_default();
        format!("manual_{teacher}_{timestamp}")
    });
    let teacher_model = if teacher == "nn_root" {
        std::env::var("HTMF_TEACHER_MODEL").ok()
    } else {
        None
    };

    TeacherMetadata {
        teacher,
        teacher_playouts: nplayouts,
        model_prior_weight,
        run_id,
        teacher_model,
    }
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

/// Extract MCTS policy from tree node visit counts
fn extract_policy(mcts: &MCTSBot, game: &GameState, _current_player: usize) -> Vec<f32> {
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
        // Absolute movement policy: 60 source cells × 6 directions × 7 distances = 2520 values
        let mut policy = vec![0.0f32; MOVEMENT_POLICY_SIZE];
        let mut total_visits = 0u32;

        if let Some(children) = mcts.root.children.get() {
            for (mov, child) in children {
                if let Move::Move((src, dst)) = mov {
                    let idx = move_to_policy_index(&Move::Move((*src, *dst)), false);
                    let (_, visits) = child.rewards_visits.get();
                    policy[idx] = visits as f32;
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

fn play_game(
    nplayouts: usize,
    nn: Option<Arc<NeuralNet>>,
    metadata: &TeacherMetadata,
) -> GameResult {
    let mut game = GameState::new_two_player(&mut rand::rng());
    let mut bots: Vec<MCTSBot> = (0..NUM_PLAYERS)
        .map(|i| {
            // Always use the production PUCT path. If an NN is present, apply it
            // only to the root before each search, matching browser serving.
            MCTSBot::new(game.clone(), Player { id: i })
        })
        .collect();

    // Store intermediate samples (without final value)
    struct PendingSample {
        features: Vec<f32>,
        policy: Vec<f32>,
        player: usize,
        turn: usize,
        is_drafting: bool,
    }
    let mut pending_samples: Vec<PendingSample> = vec![];
    let mut num_moves = 0;

    while let Some(p) = game.active_player() {
        if let Some(nn) = &nn {
            match nn.predict(&game, p.id) {
                Ok(output) => bots[p.id].update_root_priors_from_logits(&output.policy_logits),
                Err(err) => eprintln!("Model prior inference failed; using uniform root: {err}"),
            }
        }

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
            turn: num_moves,
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

    // Compute ownership targets from final game state
    // Each cell is owned by the player who claimed it (or 2 if unclaimed)
    let mut ownership = vec![2u8; NUM_CELLS]; // Default: unclaimed
    for cell in game.board.claimed[0].into_iter() {
        ownership[cell as usize] = 0;
    }
    for cell in game.board.claimed[1].into_iter() {
        ownership[cell as usize] = 1;
    }

    // Compute score difference targets (per-player perspective)
    let score_diffs: [i32; 2] = [
        scores[0] as i32 - scores[1] as i32,
        scores[1] as i32 - scores[0] as i32,
    ];

    // Convert pending samples to final training samples
    let samples: Vec<TrainingSample> = pending_samples
        .into_iter()
        .map(|s| {
            let score_diff_bin = (score_diffs[s.player] + 92) as u8;
            TrainingSample {
                policy_version: POLICY_VERSION,
                teacher: metadata.teacher.clone(),
                teacher_playouts: metadata.teacher_playouts,
                model_prior_weight: metadata.model_prior_weight,
                run_id: metadata.run_id.clone(),
                teacher_model: metadata.teacher_model.clone(),
                features: s.features,
                policy: s.policy,
                value: values[s.player],
                player: s.player,
                turn: s.turn,
                is_drafting: s.is_drafting,
                ownership: ownership.clone(),
                score_diff: score_diff_bin,
            }
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
    let metadata = TeacherMetadata {
        teacher: "uniform".to_owned(),
        teacher_playouts: nplayouts,
        model_prior_weight: 0.0,
        run_id: "test_uniform".to_owned(),
        teacher_model: None,
    };
    let result = play_game(nplayouts, None, &metadata);

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
        first.policy_version, POLICY_VERSION,
        "Samples should declare the current policy encoding version"
    );
    assert_eq!(first.teacher, "uniform");
    assert_eq!(first.teacher_playouts, nplayouts);
    assert_eq!(first.model_prior_weight, 0.0);
    assert_eq!(first.run_id, "test_uniform");
    assert!(first.teacher_model.is_none());
    assert_eq!(
        first.features.len(),
        8 * NUM_CELLS,
        "Features should be 8 channels x 60 cells"
    );
    assert_eq!(first.turn, 0, "First sample should be turn 0");

    // First 8 moves are drafting
    for (turn, sample) in result.samples.iter().take(8).enumerate() {
        assert!(
            sample.is_drafting,
            "First 8 samples should be drafting phase"
        );
        assert_eq!(sample.turn, turn, "Sample turn should match move number");
        assert_eq!(
            sample.policy.len(),
            NUM_CELLS,
            "Drafting policy should be 60 values"
        );
    }

    // After drafting, policy should be movement (2520 values - absolute source-cell encoding)
    if result.samples.len() > 8 {
        let movement_sample = &result.samples[8];
        assert!(
            !movement_sample.is_drafting,
            "Sample 9+ should be movement phase"
        );
        assert_eq!(
            movement_sample.policy.len(),
            MOVEMENT_POLICY_SIZE,
            "Movement policy should be 2520 values (60 cells × 6 directions × 7 distances)"
        );
        assert_eq!(movement_sample.turn, 8, "First movement sample should be turn 8");
    }

    // Values should be valid (0.0, 0.5, or 1.0)
    for sample in &result.samples {
        assert!(
            sample.value == 0.0 || sample.value == 0.5 || sample.value == 1.0,
            "Value should be 0, 0.5, or 1"
        );
    }
}

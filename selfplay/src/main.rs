extern crate rand;
extern crate rayon;
extern crate serde;
extern crate serde_json;

extern crate htmf;
extern crate htmf_bots;

use std::sync::mpsc;

use rayon::prelude::*;
use serde::Serialize;

use htmf::board::*;
use htmf::game::*;
use htmf::NUM_CELLS;
use htmf_bots::mctsbot::*;

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
    /// For movement: 60*60 = 3600 values (src*60 + dst)
    pub policy: Vec<f32>,
    /// Game outcome from current player's perspective: 1.0 = win, 0.5 = draw, 0.0 = loss
    pub value: f32,
    /// Current player (0 or 1)
    pub player: usize,
    /// Whether this is drafting phase
    pub is_drafting: bool,
}

fn main() {
    let ntrials: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);
    let nplayouts: usize = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(14_000);

    eprintln!(
        "Running {} games with {} playouts per move",
        ntrials, nplayouts
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
        .map(|_| play_game(nplayouts))
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

/// Extract MCTS policy from tree node visit counts
fn extract_policy(mcts: &MCTSBot, game: &GameState) -> Vec<f32> {
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
        // Movement policy: 60*60 = 3600 values (src * 60 + dst)
        let mut policy = vec![0.0f32; NUM_CELLS * NUM_CELLS];
        let mut total_visits = 0u32;

        if let Some(children) = mcts.root.children.get() {
            for (mov, child) in children {
                if let Move::Move((src, dst)) = mov {
                    let (_, visits) = child.rewards_visits.get();
                    let idx = (*src as usize) * NUM_CELLS + (*dst as usize);
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

fn play_game(nplayouts: usize) -> GameResult {
    let mut game = GameState::new_two_player(&mut rand::rng());
    let mut bots: Vec<MCTSBot> = (0..NUM_PLAYERS)
        .map(|i| MCTSBot::new(game.clone(), Player { id: i }))
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
        let policy = extract_policy(&bots[p.id], &game);
        let is_drafting = !game.finished_drafting();

        pending_samples.push(PendingSample {
            features,
            policy,
            player: p.id,
            is_drafting,
        });

        // Take the action
        let action = bots[p.id].take_action();
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
    let result = play_game(nplayouts);

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

    // After drafting, policy should be movement (3600 values)
    if result.samples.len() > 8 {
        let movement_sample = &result.samples[8];
        assert!(
            !movement_sample.is_drafting,
            "Sample 9+ should be movement phase"
        );
        assert_eq!(
            movement_sample.policy.len(),
            NUM_CELLS * NUM_CELLS,
            "Movement policy should be 3600 values"
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

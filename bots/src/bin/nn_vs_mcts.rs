//! Benchmark: Neural Network guided MCTS vs Traditional MCTS
//!
//! The NN bot uses PUCT selection with NN policy priors and random rollouts.
//! This provides a strong baseline that can be incrementally improved through training.
//!
//! Usage: nn_vs_mcts [num_games] [num_playouts] [--uniform]
//!   --uniform: Use uniform priors instead of trained NN (for baseline comparison)

use std::sync::Arc;
use std::time::{Duration, Instant};

use htmf::board::Player;
use htmf::game::GameState;
use htmf_bots::{MCTSBot, NeuralNet};
use rand::prelude::*;
use rayon::prelude::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Check for --uniform flag
    let use_uniform = args.iter().any(|a| a == "--uniform");
    let numeric_args: Vec<&String> = args
        .iter()
        .skip(1)
        .filter(|a| !a.starts_with("--"))
        .collect();

    let num_games: usize = numeric_args
        .first()
        .and_then(|s| s.parse().ok())
        .unwrap_or(40);
    let num_playouts: usize = numeric_args
        .get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(400);

    // Load neural network unless uniform mode
    let nn: Option<Arc<NeuralNet>> = if use_uniform {
        eprintln!("Using uniform priors (PUCT without trained NN)");
        None
    } else {
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
    };

    let mode_name = if nn.is_some() {
        "NN-guided PUCT"
    } else {
        "PUCT (uniform priors)"
    };

    println!("{} vs Traditional MCTS", mode_name);
    println!("========================================");
    println!("Games: {}", num_games);
    println!("Playouts per move: {}", num_playouts);
    println!();

    // Result type including timing
    struct GameResult {
        game_num: usize,
        nn_player: usize,
        nn_score: usize,
        mcts_player: usize,
        mcts_score: usize,
        move_count: usize,
        result: usize, // 0=nn_win, 1=mcts_win, 2=draw
        nn_time: Duration,
        mcts_time: Duration,
    }

    // Run games in parallel
    let results: Vec<GameResult> = (0..num_games)
        .into_par_iter()
        .map(|game_num| {
            // Alternate who goes first
            let nn_player = game_num % 2;
            let mcts_player = 1 - nn_player;

            let mut rng = StdRng::seed_from_u64(game_num as u64);
            let mut game = GameState::new_two_player(&mut rng);

            let mut nn_bot = MCTSBot::with_neural_net(
                game.clone(),
                Player { id: nn_player },
                nn.clone(),
            );
            let mut mcts_bot = MCTSBot::new(game.clone(), Player { id: mcts_player });

            let mut move_count = 0;
            let mut nn_time = Duration::ZERO;
            let mut mcts_time = Duration::ZERO;

            while let Some(p) = game.active_player() {
                let action = if p.id == nn_player {
                    let start = Instant::now();
                    for _ in 0..num_playouts {
                        nn_bot.playout();
                    }
                    let result = nn_bot.take_action();
                    nn_time += start.elapsed();
                    result
                } else {
                    let start = Instant::now();
                    for _ in 0..num_playouts {
                        mcts_bot.playout();
                    }
                    let result = mcts_bot.take_action();
                    mcts_time += start.elapsed();
                    result
                };

                game.apply_action(&action).unwrap();
                nn_bot.update(&game);
                mcts_bot.update(&game);
                move_count += 1;
            }

            let scores = game.get_scores();
            let nn_score = scores[nn_player];
            let mcts_score = scores[mcts_player];

            let result = if nn_score > mcts_score {
                0 // NN wins
            } else if mcts_score > nn_score {
                1 // MCTS wins
            } else {
                2 // Draw
            };

            GameResult {
                game_num,
                nn_player,
                nn_score,
                mcts_player,
                mcts_score,
                move_count,
                result,
                nn_time,
                mcts_time,
            }
        })
        .collect();

    // Print results in order and tally
    let mut nn_wins = 0;
    let mut mcts_wins = 0;
    let mut draws = 0;
    let mut total_nn_time = Duration::ZERO;
    let mut total_mcts_time = Duration::ZERO;

    let mut sorted_results = results;
    sorted_results.sort_by_key(|r| r.game_num);

    for r in &sorted_results {
        let result_str = match r.result {
            0 => { nn_wins += 1; "NN wins" }
            1 => { mcts_wins += 1; "MCTS wins" }
            _ => { draws += 1; "Draw" }
        };
        total_nn_time += r.nn_time;
        total_mcts_time += r.mcts_time;

        println!(
            "Game {:2}: NN(P{})={:2} vs MCTS(P{})={:2} in {:2} moves - {}",
            r.game_num + 1,
            r.nn_player,
            r.nn_score,
            r.mcts_player,
            r.mcts_score,
            r.move_count,
            result_str
        );
    }

    println!();
    println!("Results:");
    println!("  NN wins:   {} ({:.1}%)", nn_wins, 100.0 * nn_wins as f64 / num_games as f64);
    println!("  MCTS wins: {} ({:.1}%)", mcts_wins, 100.0 * mcts_wins as f64 / num_games as f64);
    println!("  Draws:     {} ({:.1}%)", draws, 100.0 * draws as f64 / num_games as f64);

    println!();
    println!("Thinking time:");
    println!("  NN total:   {:.2}s", total_nn_time.as_secs_f64());
    println!("  MCTS total: {:.2}s", total_mcts_time.as_secs_f64());
    println!("  NN/MCTS ratio: {:.2}x", total_nn_time.as_secs_f64() / total_mcts_time.as_secs_f64());
}

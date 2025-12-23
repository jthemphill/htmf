//! Benchmark: Neural Network guided MCTS vs Traditional MCTS
//!
//! The NN bot uses PUCT selection with NN policy priors and random rollouts.
//! This provides a strong baseline that can be incrementally improved through training.
//!
//! Usage: nn_vs_mcts [num_games] [num_playouts] [--uniform]
//!   --uniform: Use uniform priors instead of trained NN (for baseline comparison)

use std::sync::Arc;

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

    // Run games in parallel
    let results: Vec<(usize, usize, usize, usize, usize, usize, usize)> = (0..num_games)
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
            while let Some(p) = game.active_player() {
                let action = if p.id == nn_player {
                    for _ in 0..num_playouts {
                        nn_bot.playout();
                    }
                    nn_bot.take_action()
                } else {
                    for _ in 0..num_playouts {
                        mcts_bot.playout();
                    }
                    mcts_bot.take_action()
                };

                game.apply_action(&action).unwrap();
                nn_bot.update(&game);
                mcts_bot.update(&game);
                move_count += 1;
            }

            let scores = game.get_scores();
            let nn_score = scores[nn_player];
            let mcts_score = scores[mcts_player];

            // Return: (game_num, nn_player, nn_score, mcts_player, mcts_score, move_count, result)
            // result: 0=nn_win, 1=mcts_win, 2=draw
            let result = if nn_score > mcts_score {
                0 // NN wins
            } else if mcts_score > nn_score {
                1 // MCTS wins
            } else {
                2 // Draw
            };

            (game_num, nn_player, nn_score, mcts_player, mcts_score, move_count, result)
        })
        .collect();

    // Print results in order and tally
    let mut nn_wins = 0;
    let mut mcts_wins = 0;
    let mut draws = 0;

    let mut sorted_results = results.clone();
    sorted_results.sort_by_key(|r| r.0);

    for (game_num, nn_player, nn_score, mcts_player, mcts_score, move_count, result) in sorted_results {
        let result_str = match result {
            0 => { nn_wins += 1; "NN wins" }
            1 => { mcts_wins += 1; "MCTS wins" }
            _ => { draws += 1; "Draw" }
        };

        println!(
            "Game {:2}: NN(P{})={:2} vs MCTS(P{})={:2} in {:2} moves - {}",
            game_num + 1,
            nn_player,
            nn_score,
            mcts_player,
            mcts_score,
            move_count,
            result_str
        );
    }

    println!();
    println!("Results:");
    println!("  NN wins:   {} ({:.1}%)", nn_wins, 100.0 * nn_wins as f64 / num_games as f64);
    println!("  MCTS wins: {} ({:.1}%)", mcts_wins, 100.0 * mcts_wins as f64 / num_games as f64);
    println!("  Draws:     {} ({:.1}%)", draws, 100.0 * draws as f64 / num_games as f64);
}

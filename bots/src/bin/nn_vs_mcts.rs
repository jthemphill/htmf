//! Benchmark: Neural Network guided MCTS vs Traditional MCTS
//!
//! The NN bot uses value network for leaf evaluation (no rollouts needed),
//! so it can search deeper with fewer playouts. We give it 1/10th the playouts
//! of traditional MCTS to test if it can compete with better guidance.

use std::sync::Arc;

use htmf::board::Player;
use htmf::game::GameState;
use htmf_bots::{MCTSBot, NeuralNet};
use rand::prelude::*;

const NUM_GAMES: usize = 40;
// Equal playouts for fair comparison
const NN_PLAYOUTS: usize = 400;
const MCTS_PLAYOUTS: usize = 400;

fn main() {
    // Load neural network
    let nn = Arc::new(
        NeuralNet::load(
            "training/artifacts/model_drafting.onnx",
            "training/artifacts/model_movement.onnx",
        )
        .expect("Failed to load neural network. Run `uv run train.py` first."),
    );

    println!("Neural Network MCTS vs Traditional MCTS");
    println!("========================================");
    println!("Games: {}", NUM_GAMES);
    println!("NN playouts: {}, MCTS playouts: {}", NN_PLAYOUTS, MCTS_PLAYOUTS);
    println!();

    let mut nn_wins = 0;
    let mut mcts_wins = 0;
    let mut draws = 0;

    for game_num in 0..NUM_GAMES {
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
                for _ in 0..NN_PLAYOUTS {
                    nn_bot.playout();
                }
                nn_bot.take_action()
            } else {
                for _ in 0..MCTS_PLAYOUTS {
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

        let result = if nn_score > mcts_score {
            nn_wins += 1;
            "NN wins"
        } else if mcts_score > nn_score {
            mcts_wins += 1;
            "MCTS wins"
        } else {
            draws += 1;
            "Draw"
        };

        println!(
            "Game {:2}: NN(P{})={:2} vs MCTS(P{})={:2} in {:2} moves - {}",
            game_num + 1,
            nn_player,
            nn_score,
            mcts_player,
            mcts_score,
            move_count,
            result
        );
    }

    println!();
    println!("Results:");
    println!("  NN wins:   {} ({:.1}%)", nn_wins, 100.0 * nn_wins as f64 / NUM_GAMES as f64);
    println!("  MCTS wins: {} ({:.1}%)", mcts_wins, 100.0 * mcts_wins as f64 / NUM_GAMES as f64);
    println!("  Draws:     {} ({:.1}%)", draws, 100.0 * draws as f64 / NUM_GAMES as f64);
}

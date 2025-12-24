//! Debug benchmark: Compare MCTS modes
//!
//! This tests:
//! 1. Pure MCTS (UCB1) vs PUCT with uniform priors - PUCT should be similar or better
//! 2. PUCT with NN priors vs PUCT with uniform priors - shows if NN is helping
//!
//! The PUCT mode uses random rollouts for evaluation (same as Pure MCTS),
//! but uses the PUCT selection formula which outperforms UCB1.

use std::sync::Arc;

use htmf::board::Player;
use htmf::game::GameState;
use htmf_bots::{MCTSBot, NeuralNet};
use rand::prelude::*;

const NUM_GAMES: usize = 40;
const PLAYOUTS: usize = 400;

fn play_match(
    game_num: usize,
    bot1_name: &str,
    bot2_name: &str,
    mut make_bot1: impl FnMut(GameState, Player) -> MCTSBot,
    mut make_bot2: impl FnMut(GameState, Player) -> MCTSBot,
) -> (i32, i32, i32) {
    let bot1_player = game_num % 2;
    let bot2_player = 1 - bot1_player;

    let mut rng = StdRng::seed_from_u64(game_num as u64);
    let mut game = GameState::new_two_player(&mut rng);

    let mut bot1 = make_bot1(game.clone(), Player { id: bot1_player });
    let mut bot2 = make_bot2(game.clone(), Player { id: bot2_player });

    let mut move_count = 0;
    while let Some(p) = game.active_player() {
        let action = if p.id == bot1_player {
            for _ in 0..PLAYOUTS {
                bot1.playout();
            }
            bot1.take_action()
        } else {
            for _ in 0..PLAYOUTS {
                bot2.playout();
            }
            bot2.take_action()
        };

        game.apply_action(&action).unwrap();
        bot1.update(&game);
        bot2.update(&game);
        move_count += 1;
    }

    let scores = game.get_scores();
    let bot1_score = scores[bot1_player];
    let bot2_score = scores[bot2_player];

    let (bot1_wins, bot2_wins, draws) = if bot1_score > bot2_score {
        (1, 0, 0)
    } else if bot2_score > bot1_score {
        (0, 1, 0)
    } else {
        (0, 0, 1)
    };

    println!(
        "Game {:2}: {}(P{})={:2} vs {}(P{})={:2} in {:2} moves",
        game_num + 1,
        bot1_name,
        bot1_player,
        bot1_score,
        bot2_name,
        bot2_player,
        bot2_score,
        move_count
    );

    (bot1_wins, bot2_wins, draws)
}

fn run_comparison(
    name: &str,
    bot1_name: &str,
    bot2_name: &str,
    make_bot1: impl Fn(GameState, Player) -> MCTSBot + Clone,
    make_bot2: impl Fn(GameState, Player) -> MCTSBot + Clone,
) {
    println!("\n{}", name);
    println!("{}", "=".repeat(name.len()));
    println!();

    let mut bot1_wins = 0;
    let mut bot2_wins = 0;
    let mut draws = 0;

    for game_num in 0..NUM_GAMES {
        let (w1, w2, d) = play_match(
            game_num,
            bot1_name,
            bot2_name,
            make_bot1.clone(),
            make_bot2.clone(),
        );
        bot1_wins += w1;
        bot2_wins += w2;
        draws += d;
    }

    println!();
    println!("Results:");
    println!(
        "  {} wins: {} ({:.1}%)",
        bot1_name,
        bot1_wins,
        100.0 * bot1_wins as f64 / NUM_GAMES as f64
    );
    println!(
        "  {} wins: {} ({:.1}%)",
        bot2_name,
        bot2_wins,
        100.0 * bot2_wins as f64 / NUM_GAMES as f64
    );
    println!(
        "  Draws: {} ({:.1}%)",
        draws,
        100.0 * draws as f64 / NUM_GAMES as f64
    );
}

fn main() {
    println!("MCTS Mode Comparison");
    println!("====================");
    println!("Playouts per move: {}", PLAYOUTS);
    println!("Games per comparison: {}", NUM_GAMES);

    // Test 1: Pure MCTS (UCB1) vs PUCT with uniform priors
    // PUCT should be similar or better than UCB1
    run_comparison(
        "Test 1: Pure MCTS (UCB1) vs PUCT (uniform priors)",
        "Pure",
        "PUCT",
        |game, player| MCTSBot::new(game, player),
        |game, player| MCTSBot::with_neural_net(game, player, None),
    );

    // Test 2: Try to load NN and compare
    match NeuralNet::load(
        "training/artifacts/model_drafting.onnx",
        "training/artifacts/model_movement.onnx",
    ) {
        Ok(nn) => {
            let nn = Arc::new(nn);

            // Test 2: PUCT with NN priors vs PUCT with uniform priors
            // If NN is trained well, NN should win
            let nn_clone = nn.clone();
            run_comparison(
                "Test 2: PUCT (NN priors) vs PUCT (uniform priors)",
                "NN",
                "Uniform",
                move |game, player| MCTSBot::with_neural_net(game, player, Some(nn_clone.clone())),
                |game, player| MCTSBot::with_neural_net(game, player, None),
            );

            // Test 3: PUCT with NN priors vs Pure MCTS
            let nn_clone2 = nn.clone();
            run_comparison(
                "Test 3: PUCT (NN priors) vs Pure MCTS (UCB1)",
                "NN",
                "Pure",
                move |game, player| MCTSBot::with_neural_net(game, player, Some(nn_clone2.clone())),
                |game, player| MCTSBot::new(game, player),
            );
        }
        Err(e) => {
            println!("\nSkipping NN tests - could not load models: {}", e);
            println!("Run `uv run training/train.py` to generate models.");
        }
    }

    println!("\n\nInterpretation Guide:");
    println!("=====================");
    println!("- PUCT should be similar or better than Pure MCTS");
    println!("- If NN >> Uniform: Neural network policy priors are helping");
    println!("- If Uniform >> NN: Neural network priors are hurting (bad training)");
}

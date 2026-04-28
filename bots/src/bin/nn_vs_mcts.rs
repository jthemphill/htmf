//! Evaluate trained policy priors against the production uniform-prior baseline.
//!
//! Usage: nn_vs_mcts [num_pairs] [num_playouts] [--uniform-vs-uniform] [--min-score SCORE]
//!   num_pairs: each seed is played twice with colors swapped
//!   --uniform-vs-uniform: sanity-check the evaluator against itself
//!   --min-score: return a failing exit code if model score is below this value

use std::process::ExitCode;
use std::sync::Arc;
use std::time::{Duration, Instant};

use htmf::board::Player;
use htmf::game::GameState;
use htmf_bots::{MCTSBot, NeuralNet};
use rand::prelude::*;
use rayon::prelude::*;

struct GameResult {
    game_num: usize,
    model_player: usize,
    model_score: usize,
    baseline_player: usize,
    baseline_score: usize,
    move_count: usize,
    result: usize, // 0=model_win, 1=baseline_win, 2=draw
    model_time: Duration,
    baseline_time: Duration,
}

fn play_game(
    game_num: usize,
    seed: u64,
    model_player: usize,
    num_playouts: usize,
    nn: Option<Arc<NeuralNet>>,
) -> GameResult {
    let baseline_player = 1 - model_player;

    let mut rng = StdRng::seed_from_u64(seed);
    let mut game = GameState::new_two_player(&mut rng);

    let mut model_bot = MCTSBot::with_neural_net(game.clone(), Player { id: model_player }, nn);
    let mut baseline_bot = MCTSBot::new(
        game.clone(),
        Player {
            id: baseline_player,
        },
    );

    let mut move_count = 0;
    let mut model_time = Duration::ZERO;
    let mut baseline_time = Duration::ZERO;

    while let Some(p) = game.active_player() {
        let action = if p.id == model_player {
            let start = Instant::now();
            for _ in 0..num_playouts {
                model_bot.playout();
            }
            let action = model_bot.take_action();
            model_time += start.elapsed();
            action
        } else {
            let start = Instant::now();
            for _ in 0..num_playouts {
                baseline_bot.playout();
            }
            let action = baseline_bot.take_action();
            baseline_time += start.elapsed();
            action
        };

        game.apply_action(&action).unwrap();
        model_bot.update(&game);
        baseline_bot.update(&game);
        move_count += 1;
    }

    let scores = game.get_scores();
    let model_score = scores[model_player];
    let baseline_score = scores[baseline_player];
    let result = if model_score > baseline_score {
        0
    } else if baseline_score > model_score {
        1
    } else {
        2
    };

    GameResult {
        game_num,
        model_player,
        model_score,
        baseline_player,
        baseline_score,
        move_count,
        result,
        model_time,
        baseline_time,
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let mut uniform_vs_uniform = false;
    let mut min_score: Option<f64> = None;
    let mut numeric_args = Vec::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--uniform-vs-uniform" => {
                uniform_vs_uniform = true;
                i += 1;
            }
            "--min-score" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("--min-score requires a numeric value");
                    return ExitCode::from(2);
                };
                match value.parse::<f64>() {
                    Ok(value) => min_score = Some(value),
                    Err(_) => {
                        eprintln!("Invalid --min-score value: {value}");
                        return ExitCode::from(2);
                    }
                }
                i += 2;
            }
            value if value.starts_with("--") => {
                eprintln!("Unknown argument: {value}");
                return ExitCode::from(2);
            }
            value => {
                numeric_args.push(value.to_owned());
                i += 1;
            }
        }
    }

    let num_pairs: usize = numeric_args
        .first()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(100);
    let num_playouts: usize = numeric_args
        .get(1)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(400);

    let nn: Option<Arc<NeuralNet>> = if uniform_vs_uniform {
        eprintln!("Running uniform-prior baseline against itself");
        None
    } else {
        eprintln!("Loading neural network from training/artifacts/model.onnx...");
        match NeuralNet::load("training/artifacts/model.onnx") {
            Ok(model) => Some(Arc::new(model)),
            Err(e) => {
                eprintln!("Failed to load neural network: {e:?}");
                return ExitCode::from(1);
            }
        }
    };

    let mode_name = if uniform_vs_uniform {
        "Uniform priors"
    } else {
        "Trained priors"
    };

    println!("{} vs Uniform-prior baseline", mode_name);
    println!("========================================");
    println!("Pairs: {}", num_pairs);
    println!("Games: {}", num_pairs * 2);
    println!("Playouts per move: {}", num_playouts);
    println!();

    let results: Vec<GameResult> = (0..num_pairs)
        .into_par_iter()
        .flat_map_iter(|pair| {
            let seed = pair as u64;
            [
                play_game(pair * 2, seed, 0, num_playouts, nn.clone()),
                play_game(pair * 2 + 1, seed, 1, num_playouts, nn.clone()),
            ]
        })
        .collect();

    let mut sorted_results = results;
    sorted_results.sort_by_key(|r| r.game_num);

    let mut model_wins = 0;
    let mut baseline_wins = 0;
    let mut draws = 0;
    let mut total_model_time = Duration::ZERO;
    let mut total_baseline_time = Duration::ZERO;

    for r in &sorted_results {
        let result_str = match r.result {
            0 => {
                model_wins += 1;
                "Model wins"
            }
            1 => {
                baseline_wins += 1;
                "Baseline wins"
            }
            _ => {
                draws += 1;
                "Draw"
            }
        };
        total_model_time += r.model_time;
        total_baseline_time += r.baseline_time;

        println!(
            "Game {:3}: Model(P{})={:2} vs Baseline(P{})={:2} in {:2} moves - {}",
            r.game_num + 1,
            r.model_player,
            r.model_score,
            r.baseline_player,
            r.baseline_score,
            r.move_count,
            result_str
        );
    }

    let total_games = sorted_results.len();
    let decisive_games = model_wins + baseline_wins;
    let score = (model_wins as f64 + 0.5 * draws as f64) / total_games as f64;

    println!();
    println!("Results:");
    println!(
        "  Model wins:    {} ({:.1}%)",
        model_wins,
        100.0 * model_wins as f64 / total_games as f64
    );
    println!(
        "  Baseline wins: {} ({:.1}%)",
        baseline_wins,
        100.0 * baseline_wins as f64 / total_games as f64
    );
    println!(
        "  Draws:         {} ({:.1}%)",
        draws,
        100.0 * draws as f64 / total_games as f64
    );
    println!("  Score:         {:.3}", score);
    println!("  Decisive:      {}", decisive_games);
    if let Some(min_score) = min_score {
        let gate = if score >= min_score { "PASS" } else { "FAIL" };
        println!("  Gate:          {} (min score {:.3})", gate, min_score);
    }

    println!();
    println!("Thinking time:");
    println!("  Model total:    {:.2}s", total_model_time.as_secs_f64());
    println!(
        "  Baseline total: {:.2}s",
        total_baseline_time.as_secs_f64()
    );
    println!(
        "  Model/Baseline ratio: {:.2}x",
        total_model_time.as_secs_f64() / total_baseline_time.as_secs_f64()
    );

    if min_score.is_some_and(|min_score| score < min_score) {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

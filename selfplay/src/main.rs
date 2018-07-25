extern crate rand;
extern crate rayon;

extern crate htmf;
extern crate htmf_bots;

use rayon::prelude::*;

use htmf::board::*;
use htmf::game::*;
use htmf_bots::mctsbot::*;
use htmf_bots::randombot::*;

fn main() {
    let verbose = std::env::args().into_iter().any(|arg| arg == "-v");
    let trials = 1000;
    let results : Vec<(i32, Vec<GameState>)> = (0..trials).into_par_iter()
        .map(|_| play_game(verbose))
        .collect();

    let mcts_wins : usize = results.iter()
        .map(|(winner, _)|
            match winner {
                0 => 0,
                1 => 1,
                _ => panic!("Expected winning player to be 0 or 1, got {}", winner)
            }).sum();
    for (winner, gamestates) in results {
         if verbose && !gamestates.is_empty() {
            for g in gamestates {
                println!("game: {}", g);
            }
            println!("winner: {}", winner);
        }
   }
   println!("{} / {} wins.", mcts_wins, trials);
}

fn play_game(verbose: bool) -> (i32, Vec<GameState>) {
    let seed = rand::random();
    let mut game = GameState::new_two_player(seed);
    let mut random = RandomBot::new(&game, Player { id: 0 });
    let mut mcts = MCTSBot::new(&game, Player { id: 1 });

    let mut logged_states = vec![];

    while let Some(p) = game.active_player() {
        let action = match p.id {
            0 => random.take_action(),
            1 => mcts.take_action(),
            _ => panic!("How many players are in this game?"),
        };

        match action {
            Action::Move(src, dst) => {
                if let Err(_) = game.move_penguin(src, dst) {
                    panic!(format!("Player {} made an illegal move", p.id));
                }
            }
            Action::Place(dst) => {
                if let Err(_) = game.place_penguin(dst) {
                    panic!(format!("Player {} made an illegal move", p.id));
                }
                if game.board.is_cut_cell(dst) {
                    game.board.prune();
                }
                game.board.reap();
            }
            _ => panic!("Unexpected action received"),
        };

        let logging_roll : f64 = rand::random();
        if verbose && logging_roll < 0.01 {
            logged_states.push(game.clone());
        }

        random.update(&game);
        mcts.update(&game);
    }
    let (winner, _score) = game
        .get_scores()
        .into_iter()
        .enumerate()
        .max_by(|(_, score1), (_, score2)| score1.cmp(score2))
        .unwrap();
    (winner as i32, logged_states)
}

extern crate rand;
extern crate rayon;

extern crate htmf;
extern crate htmf_bots;

use std::sync::mpsc;

use rand::prelude::*;
use rayon::prelude::*;

use htmf::board::*;
use htmf::game::*;
use htmf_bots::mctsbot::*;
use htmf_bots::randombot::*;

const RANDOM_PLAYER: usize = 0;
const MCTS_PLAYER: usize = 1;

fn main() {
    let ntrials = 10;
    let nplayouts = 100;
    let verbose = std::env::args().any(|arg| arg == "-v");

    let (logger_tx, logger_rx) = mpsc::channel();
    std::thread::spawn(move || loop {
        let res: Result<(usize, Vec<GameState>), ()> = logger_rx.recv().unwrap();
        if let Ok((winner, gamestates)) = res {
            if !gamestates.is_empty() {
                for g in gamestates {
                    println!("game: {}", g);
                }
                println!("winner: {}", winner);
            }
        } else {
            break;
        }
    });

    let mcts_wins: usize = (0..ntrials)
        .into_par_iter()
        .map(|_| play_game(nplayouts, verbose))
        .map_with(
            mpsc::Sender::clone(&logger_tx),
            |logger_tx, (winner, gamestates)| {
                if let Err(mpsc::SendError(_)) = logger_tx.send(Ok((winner, gamestates))) {
                    // Do nothing
                };
                match winner {
                    0 => 0,
                    1 => 1,
                    _ => panic!("Expected winning player to be 0 or 1, got {}", winner),
                }
            },
        )
        .sum();

    // Stop the logger
    logger_tx.send(Err(())).unwrap();

    println!("{} / {} wins.", mcts_wins, ntrials);
}

fn play_game(nplayouts: usize, verbose: bool) -> (usize, Vec<GameState>) {
    let mut game = GameState::new_two_player::<StdRng>(&mut SeedableRng::from_entropy());
    let mut random = RandomBot::new(game.clone(), Player { id: RANDOM_PLAYER });
    let mut mcts = MCTSBot::new(game.clone(), Player { id: MCTS_PLAYER });

    let mut logged_states = vec![];

    while let Some(p) = game.active_player() {
        let action = match p.id {
            RANDOM_PLAYER => random.take_action(),
            MCTS_PLAYER => {
                for _ in 0..nplayouts {
                    mcts.playout();
                }
                mcts.take_action()
            }
            _ => panic!("How many players are in this game?"),
        };

        match action {
            Action::Move(src, dst) => {
                if game.move_penguin(src, dst).is_err() {
                    panic!("Illegal move");
                }
            }
            Action::Place(dst) => {
                if game.place_penguin(dst).is_err() {
                    panic!("Illegal placement");
                }
                if game.board.is_cut_cell(dst) {
                    game.board.prune();
                }
                game.board.reap();
            }
            _ => panic!("Unexpected action received"),
        };

        let logging_roll: f64 = rand::random();
        if verbose && logging_roll < 0.01 {
            logged_states.push(game.clone());
        }

        random.update(&game);
        mcts.update(&game);
        assert!(mcts.root.game.state == game);
    }
    let (winner, _score) = game
        .get_scores()
        .into_iter()
        .enumerate()
        .max_by(|(_, score1), (_, score2)| score1.cmp(score2))
        .unwrap();
    (winner, logged_states)
}

#[test]
fn test_mcts_beats_random() {
    let ntrials = 10;
    let nplayouts = 100;
    let verbose = false;

    let mcts_wins: usize = (0..ntrials)
        .into_par_iter()
        .map(|_| {
            let (winner, _) = play_game(nplayouts, verbose);
            match winner {
                RANDOM_PLAYER => 0,
                MCTS_PLAYER => 1,
                _ => panic!("Expected winning player to be 0 or 1, got {}", winner),
            }
        })
        .sum();
    assert!(
        mcts_wins > ntrials * 3 / 4,
        "Only won {} out of {} games",
        mcts_wins,
        ntrials
    );
}

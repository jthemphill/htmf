extern crate rand;

extern crate htmf;
extern crate htmf_bots;

use self::htmf::board::*;
use self::htmf::game::*;
use self::htmf_bots::mctsbot::*;
use self::htmf_bots::randombot::*;

fn main() {
    let trials = 1000;
    let mut mcts_wins = 0;
    for _ in 0..trials {
        let winner = play_game();
        match winner {
            0 => {}
            1 => {
                mcts_wins += 1;
            }
            _ => panic!("Wrong player as winner"),
        }
    }
    println!("{} / {} wins.", mcts_wins, trials);
}

fn play_game() -> i32 {
    let seed = rand::random();
    let mut game = GameState::new_two_player(&[seed]);
    let mut random = RandomBot::new(&game, Player { id: 0 });
    let mut mcts = MCTSBot::new(&game, Player { id: 1 });

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
            }
            _ => panic!("Unexpected action received"),
        };

        if game.finished_drafting() {
            game.board.prune();
            game.board.reap();
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
    winner as i32
}

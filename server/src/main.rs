extern crate rand;
extern crate tungstenite;

extern crate htmf;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod protocol;
mod session;

use rand::Rng;

use tungstenite::server::accept;
use tungstenite::Message;

use htmf::game::{Action, GameState};
use htmf::json::*;

use session::Session;

fn main() {
    let server = std::net::TcpListener::bind("127.0.0.1:2794").unwrap();
    for stream in server.incoming() {
        std::thread::spawn(move || {
            let mut websocket = accept(stream.unwrap()).unwrap();
            let mut rng = rand::thread_rng();
            let mut session = Session::new(GameState::new_two_player(rng.gen()));

            let mut init_json = GameStateJSON::from(&session.game.board);
            add_possible_moves(&mut init_json);
            websocket
                .write_message(Message::Text(String::from(&init_json)))
                .unwrap();

            while let Ok(request_msg) = websocket.read_message() {
                if let Message::Text(request_str) = request_msg {
                    let mut response_json = get_response(&mut session, &request_str);
                    add_possible_moves(&mut response_json);
                    if websocket
                        .write_message(Message::Text(String::from(&response_json)))
                        .is_err()
                    {
                        break;
                    }
                }
            }
        });
    }
}

fn add_possible_moves(game_json: &mut htmf::json::GameStateJSON) {
    if game_json.board.possible_moves.is_some() {
        return;
    }
    game_json.board.possible_moves = match game_json.mode_type {
        GameModeType::Drafting => {
            Some(
                game_json
                    .board
                    .fish
                    .iter()
                    .enumerate()
                    .filter(|(_, &fish)| fish == 1)
                    .map(|(idx, _)| idx as u8)
                    .collect(),
            )
        }
        GameModeType::Playing => {
            if let Some(p) = game_json.active_player {
                Some(game_json.board.penguins[p].clone())
            } else {
                None
            }
        }
    }
}

fn get_response(session: &mut Session, action_str: &str) -> htmf::json::GameStateJSON {
    let action = match protocol::action_from_str(action_str) {
        Some(a) => a,
        None => {
            let mut game_json = GameStateJSON::from(&session.game);
            game_json.last_move_valid = false;
            return game_json;
        }
    };
    match action {
        Action::Selection(cell_idx) => {
            let board = &session.game.board;
            let mut board_json = BoardJSON::from(board);
            board_json.possible_moves = Some(board.moves(cell_idx).into_iter().collect());
            let mut game_json = GameStateJSON::from(&session.game);
            game_json.board = board_json;
            game_json
        }
        _ => {
            let res = session.apply_action(&action);
            let mut game_json = GameStateJSON::from(&session.game);
            if res.is_err() {
                game_json.last_move_valid = false;
            }
            game_json
        }
    }
}

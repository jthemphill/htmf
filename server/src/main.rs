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

            let init_msg = String::from(&GameStateJSON::from(&session.game.board));
            websocket.write_message(Message::Text(init_msg)).unwrap();

            loop {
                let request_msg = websocket.read_message().unwrap();

                match request_msg {
                    Message::Text(request_str) => {
                        let response_str = get_response(&mut session, &request_str);
                        websocket
                            .write_message(Message::Text(response_str))
                            .unwrap();
                    }
                    _ => {}
                };
            }
        });
    }
}

fn get_response(session: &mut Session, action_str: &str) -> String {
    let action = match protocol::action_from_str(action_str) {
        Some(a) => a,
        None => {
            let mut game_json = GameStateJSON::from(&session.game);
            game_json.last_move_valid = false;
            return String::from(&game_json);
        }
    };
    match action {
        Action::Selection(cell_idx) => {
            let board = &session.game.board;
            let mut board_json = BoardJSON::from(board);
            board_json.possible_moves = Some(board.moves(cell_idx).into_iter().collect());
            let mut game_json = GameStateJSON::from(&session.game);
            game_json.board = board_json;
            String::from(&game_json)
        }
        _ => {
            let res = session.apply_action(&action);
            let mut game_json = GameStateJSON::from(&session.game);
            if res.is_err() {
                game_json.last_move_valid = false;
            }
            String::from(&game_json)
        }
    }
}

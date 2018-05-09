extern crate futures;
extern crate rand;
extern crate tokio_core;
extern crate websocket;

extern crate htmf;

#[macro_use]
extern crate serde_derive;

mod protocol;
mod session;

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;

use rand::Rng;

use websocket::{Message, OwnedMessage};
use websocket::async::Server;
use websocket::server::InvalidConnection;

use tokio_core::reactor::{Handle, Core};
use futures::{Future, Sink, Stream};

use htmf::game::{Action, GameState};

use session::Session;

fn spawn_future<F, I, E>(f: F, desc: &'static str, handle: &Handle)
	where F: Future<Item = I, Error = E> + 'static,
		  E: Debug
{
	handle.spawn(f.map_err(move |e| println!("{}: '{:?}'", desc, e))
		         .map(move |_| println!("{}: Finished.", desc)));
}

fn main() {

	let mut core = Core::new().unwrap();
	let handle = core.handle();

	let server = Server::bind("127.0.0.1:2794", &handle).unwrap();

    let sessions = Rc::new(RefCell::new(HashMap::new()));
	let f = server.incoming()
        // we don't wanna save the stream if it drops
        .map_err(|InvalidConnection { error, .. }| error)
        .for_each(move |(upgrade, addr)| {
            let sessions_init = Rc::clone(&sessions);
            let sessions_update = Rc::clone(&sessions);
            let f = upgrade.accept()
                .and_then(move |(s, _)| {
                    let mut rng = rand::thread_rng();
                    let game_state = GameState::new_two_player(&[rng.gen(); 1]);

                    let mut sessions = sessions_init.borrow_mut();
                    sessions.insert(
                        addr,
                        Session::new(game_state),
                    );

                    let session = sessions.get(&addr).unwrap();
                    println!("Started a connection");
                    s.send(
                        Message::text(
                            protocol::init_with_board(&session.game.board)
                        ).into()
                    )
                })
                .and_then(move |s| {
                    let (sink, stream) = s.split();
                    stream
                    .take_while(|m| Ok(!m.is_close()))
                    .filter_map(move |m| {
                        // println!("Message from Client: {:?}", m);
                        let mut session = {
                            let sessions = sessions_update.borrow();
                            sessions
                                .get(&addr)
                                .unwrap()
                                .clone()
                        };
                        match m {
                            OwnedMessage::Ping(p) => Some(
                                OwnedMessage::Pong(p)
                            ),
                            OwnedMessage::Pong(_) => None,
                            OwnedMessage::Text(request_str) => {
                                let response_str = get_response(
                                    &mut session,
                                    &request_str,
                                );
                                sessions_update.borrow_mut()
                                    .insert(addr, session.clone());
                                Some(OwnedMessage::Text(response_str))
                            },
                            _ => None,
                        }
                    })
                    .forward(sink)
                    .and_then(|(_, sink)| {
                        sink.send(OwnedMessage::Close(None))
                    })
                });

            spawn_future(f, "Client Status", &handle);
            Ok(())
        });

	core.run(f).unwrap();
}

fn get_response(session: &mut Session, action_str: &str) -> String {
    let action = match protocol::action_from_str(action_str) {
        Some(a) => a,
        None => {
            let mut game_json = protocol::GameStateJSON::from_game(&session.game);
            game_json.last_move_valid = false;
            return game_json.to_string();
        },
    };
    match action {
        Action::Selection(cell_idx) => {
            let board = &session.game.board;
            let possible_moves = board.moves(cell_idx);
            let mut board_json = protocol::BoardJSON::from_board(board);
            for i in possible_moves {
                board_json.possible_moves[i] = true;
            }
            let mut game_json = protocol::GameStateJSON::from_game(&session.game);
            game_json.board = board_json;
            game_json.to_string()
        },
        _ => {
            let res = session.apply_action(&action);
            let mut game_json = protocol::GameStateJSON::from_game(&session.game);
            if res.is_err() {
                game_json.last_move_valid = false;
            }
            game_json.to_string()
        },
    }
}

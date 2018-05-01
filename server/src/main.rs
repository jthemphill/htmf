#![allow(unused_imports, unused_variables)]

extern crate futures;
extern crate rand;
extern crate tokio_core;
extern crate websocket;

extern crate htmf;

#[macro_use]
extern crate serde_derive;

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::rc::Rc;

use rand::Rng;

use websocket::{Message, OwnedMessage};
use websocket::async::Server;
use websocket::server::InvalidConnection;

use tokio_core::reactor::{Handle, Core};
use futures::{Future, Sink, Stream};

use htmf::board::{Board, Player};
use htmf::game::{Action, GameState};

mod protocol;

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

    let game_states = Rc::new(RefCell::new(HashMap::new()));
	let f = server.incoming()
        // we don't wanna save the stream if it drops
        .map_err(|InvalidConnection { error, .. }| error)
        .for_each(move |(upgrade, addr)| {
            let game_states_init = Rc::clone(&game_states);
            let game_states_update = Rc::clone(&game_states);
            let f = upgrade.accept()
                .and_then(move |(s, _)| {
                    let mut rng = rand::thread_rng();
                    let game_state = GameState::new_two_player(&[rng.gen(); 1]);

                    let mut states = game_states_init.borrow_mut();
                    states.insert(
                        addr,
                        game_state,
                    );

                    let g = states.get(&addr).unwrap();
                    println!("Started a connection");
                    s.send(
                        Message::text(
                            protocol::init_with_board(
                                &g.board
                            )
                        ).into()
                    )
                })
                .and_then(move |s| {
                    let (sink, stream) = s.split();
                    stream
                    .take_while(|m| Ok(!m.is_close()))
                    .filter_map(move |m| {
                        // println!("Message from Client: {:?}", m);
                        let mut game = {
                            let states = game_states_update.borrow();
                            states
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
                                    &mut game,
                                    &request_str
                                );
                                game_states_update.borrow_mut()
                                    .insert(addr, game);
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

fn get_response(game: &mut GameState, action_str: &str) -> String {
    let action = match protocol::action_from_str(action_str) {
        Some(a) => a,
        None => {
            let mut game_json = protocol::GameStateJSON::from_game(&game);
            game_json.last_move_valid = false;
            return game_json.to_string();
        },
    };
    match action {
        Action::Move(src, dst) => {
            let res = game.move_penguin(src, dst);
            let mut game_json = protocol::GameStateJSON::from_game(&game);
            if res.is_err() {
                game_json.last_move_valid = false;
            }
            game_json.to_string()
        },
        Action::Place(cell_idx) => {
            let res = game.place_penguin(cell_idx);
            let mut game_json = protocol::GameStateJSON::from_game(&game);
            if res.is_err() {
                game_json.last_move_valid = false;
            }
            game_json.to_string()
        },
        Action::Selection(cell_idx) => {
            let board = &game.board;
            let cell = board.cells[cell_idx];
            let possible_moves = board.moves(cell_idx);
            let mut board_json = protocol::BoardJSON::from_board(board);
            for i in possible_moves {
                board_json.possible_moves[i] = true;
            }
            let mut game_json = protocol::GameStateJSON::from_game(&game);
            game_json.board = board_json;
            game_json.to_string()
        },
        Action::Setup(new_game_state) => {
            *game = new_game_state;
            protocol::GameStateJSON::from_game(&game).to_string()
        },
    }
}

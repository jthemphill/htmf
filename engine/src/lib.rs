extern crate arrayvec;
extern crate rand;

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

pub const EVEN_ROW_LEN: usize = 7;
pub const ODD_ROW_LEN: usize = 8;
pub const NUM_CELLS: usize = EVEN_ROW_LEN * (NUM_ROWS / 2) + ODD_ROW_LEN * (NUM_ROWS / 2);
pub const NUM_ROWS: usize = 8;
pub const NUM_ONE_FISH: usize = 30;
pub const NUM_TWO_FISH: usize = 20;
pub const NUM_THREE_FISH: usize = 10;
pub const NUM_FISH: usize = NUM_ONE_FISH + NUM_TWO_FISH + NUM_THREE_FISH;
pub const MOVE_ARRAY_SIZE: usize = 100;

pub mod board;
pub mod cellset;
pub mod errors;
pub mod game;
mod hex;
pub mod json;

extern crate arrayvec;
extern crate rand;

pub const EVEN_ROW_LEN: usize = 7;
pub const ODD_ROW_LEN: usize = 8;
pub const NUM_CELLS: usize = (EVEN_ROW_LEN * (NUM_ROWS / 2) + ODD_ROW_LEN * (NUM_ROWS / 2));
pub const NUM_ROWS: usize = 8;
pub const NUM_ONE_FISH: usize = 30;
pub const NUM_TWO_FISH: usize = 20;
pub const NUM_THREE_FISH: usize = 10;
pub const NUM_FISH: usize = NUM_ONE_FISH + NUM_TWO_FISH + NUM_THREE_FISH;

pub mod board;
pub mod errors;
pub mod game;
pub mod cellset;
mod hex;

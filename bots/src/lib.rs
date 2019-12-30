extern crate arrayvec;
extern crate rand;

extern crate htmf;

pub mod mctsbot;
pub mod minimaxbot;
pub mod randombot;

pub use mctsbot::MCTSBot;
pub use minimaxbot::MinimaxBot;
pub use randombot::RandomBot;

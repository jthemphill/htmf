extern crate htmf;

pub mod mctsbot;
pub mod minimaxbot;
pub mod randombot;

mod game;

pub use mctsbot::MCTSBot;
pub use minimaxbot::MinimaxBot;
pub use randombot::RandomBot;

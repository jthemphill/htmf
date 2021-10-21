use crate::board::Player;

#[derive(Debug)]
pub struct IllegalMoveError {
    pub player: Player,
    pub message: String,
}

impl IllegalMoveError {
    pub fn new(player: Player, message: String) -> Self {
        IllegalMoveError { player, message }
    }
}

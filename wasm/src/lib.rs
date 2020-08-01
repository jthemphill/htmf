use wasm_bindgen::prelude::*;

use htmf::game::GameState;

#[wasm_bindgen]
pub struct Game {
    game: GameState,
}

#[wasm_bindgen]
impl Game {
    pub fn new() -> Self {
        Self {
            game: GameState::new_two_player(rand::random()),
        }
    }

    pub fn is_drafting(&self) -> bool {
        !self.finished_drafting()
    }

    pub fn finished_drafting(&self) -> bool {
        self.game.finished_drafting()
    }

    pub fn num_players(&self) -> u8 {
        2
    }

    pub fn active_player(&self) -> Option<u8> {
        self.game.active_player().map(|p| p.id as u8)
    }

    pub fn scores(&self) -> Vec<usize> {
        self.game.get_scores().to_vec()
    }

    pub fn turn(&self) -> usize {
        self.game.turn
    }

    pub fn num_fish(&self) -> Vec<usize> {
        (0..htmf::NUM_CELLS as u8)
            .map(|idx| self.game.board.num_fish(idx))
            .collect()
    }

    pub fn penguins(&self, player: usize) -> Vec<u8> {
        self.game.board.penguins[player].into_iter().collect()
    }

    pub fn claimed(&self, player: usize) -> Vec<u8> {
        self.game.board.claimed[player].iter().collect()
    }

    pub fn possible_moves(&self, src: u8) -> Vec<u8> {
        self.game.board.moves(src).into_iter().collect()
    }

    pub fn place_penguin(&mut self, dst: u8) -> Result<(), JsValue> {
        match self.game.place_penguin(dst) {
            Ok(()) => Ok(()),
            Err(err) => Err(JsValue::from(err.message)),
        }
    }

    pub fn move_penguin(&mut self, src: u8, dst: u8) -> Result<(), JsValue> {
        match self.game.move_penguin(src, dst) {
            Ok(()) => Ok(()),
            Err(err) => Err(JsValue::from(err.message)),
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

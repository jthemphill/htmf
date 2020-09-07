use wasm_bindgen::prelude::*;

use htmf::board::Player;
use htmf::game::GameState;
use htmf_bots::MCTSBot;

mod utils;

#[wasm_bindgen]
pub struct Game {
    bot: MCTSBot,
}

#[wasm_bindgen]
impl Game {
    pub fn new() -> Self {
        utils::set_panic_hook();
        Self {
            bot: MCTSBot::new(GameState::new_two_player(rand::random()), Player { id: 1 }),
        }
    }

    pub fn is_drafting(&self) -> bool {
        !self.finished_drafting()
    }

    pub fn finished_drafting(&self) -> bool {
        self.bot.root.state.finished_drafting()
    }

    pub fn game_over(&self) -> bool {
        self.bot.root.state.game_over()
    }

    pub fn active_player(&self) -> Option<u8> {
        self.bot.root.state.active_player().map(|p| p.id as u8)
    }

    pub fn score(&self, player: usize) -> usize {
        self.bot.root.state.get_scores()[player]
    }

    pub fn turn(&self) -> usize {
        self.bot.root.state.turn
    }

    pub fn num_fish(&self, idx: u8) -> usize {
        self.bot.root.state.board.num_fish(idx)
    }

    pub fn penguins(&self, player: usize) -> Vec<u8> {
        self.bot.root.state.board.penguins[player]
            .into_iter()
            .collect()
    }

    pub fn claimed(&self, player: usize) -> Vec<u8> {
        self.bot.root.state.board.claimed[player]
            .into_iter()
            .collect()
    }

    pub fn draftable_cells(&self) -> Vec<u8> {
        self.bot.root.state.board.fish[0]
            .exclude(self.bot.root.state.board.all_claimed_cells())
            .into_iter()
            .collect()
    }

    pub fn possible_moves(&self, src: u8) -> Vec<u8> {
        self.bot.root.state.board.moves(src).into_iter().collect()
    }

    pub fn place_penguin(&mut self, dst: u8) -> Result<(), JsValue> {
        match self.bot.root.state.place_penguin(dst) {
            Ok(()) => Ok(()),
            Err(err) => Err(JsValue::from(err.message)),
        }
    }

    pub fn move_penguin(&mut self, src: u8, dst: u8) -> Result<(), JsValue> {
        match self.bot.root.state.move_penguin(src, dst) {
            Ok(()) => Ok(()),
            Err(err) => Err(JsValue::from(err.message)),
        }
    }

    pub fn playout(&mut self) {
        if self.game_over() {
            return;
        }
        self.bot.playout()
    }

    pub fn take_action(&mut self) -> Result<(), JsValue> {
        if self.game_over() {
            return Ok(());
        }
        while self.bot.root.state.active_player() == Some(self.bot.me) {
            let action = self.bot.take_action();
            match self.bot.root.state.apply_action(&action) {
                Ok(()) => Ok(()),
                Err(err) => Err(JsValue::from(err.message)),
            }?;
        }
        Ok(())
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

use rand::prelude::*;
use wasm_bindgen::prelude::*;

use htmf::board::Player;
use htmf::game::GameState;
use htmf_bots::MCTSBot;

mod utils;

#[wasm_bindgen]
pub struct MoveInfo {
    visits: u64,
    rewards: f64,
}

#[wasm_bindgen]
impl MoveInfo {
    pub fn get_visits(&self) -> f64 {
        self.visits as f64
    }

    pub fn get_rewards(&self) -> f64 {
        self.rewards
    }
}

#[wasm_bindgen]
pub struct Game {
    bot: MCTSBot<StdRng>,
}

#[wasm_bindgen]
impl Game {
    pub fn new() -> Self {
        utils::set_panic_hook();
        Self {
            bot: MCTSBot::<StdRng>::new(
                GameState::new_two_player::<StdRng>(&mut SeedableRng::from_entropy()),
                Player { id: 1 },
                SeedableRng::from_entropy(),
            ),
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

    pub fn place_penguin(&mut self, dst: u8) -> Result<JsValue, JsValue> {
        let mut new_game = self.bot.root.state.clone();
        match new_game.place_penguin(dst) {
            Ok(()) => {
                let update_stats = self.bot.update(new_game);
                Ok(JsValue::from(format!(
                    "Old size: {}. New size: {}. Old capacity: {}. New capacity: {}",
                    update_stats.old_size,
                    update_stats.new_size,
                    update_stats.old_capacity,
                    update_stats.new_capacity
                )))
            }
            Err(err) => Err(JsValue::from(err.message)),
        }
    }

    pub fn move_penguin(&mut self, src: u8, dst: u8) -> Result<JsValue, JsValue> {
        let mut new_game = self.bot.root.state.clone();
        match new_game.move_penguin(src, dst) {
            Ok(()) => {
                let update_stats = self.bot.update(new_game);
                Ok(JsValue::from(format!(
                    "Old size: {}. New size: {}. Old capacity: {}. New capacity: {}",
                    update_stats.old_size,
                    update_stats.new_size,
                    update_stats.old_capacity,
                    update_stats.new_capacity,
                )))
            }
            Err(err) => Err(JsValue::from(err.message)),
        }
    }

    pub fn playout(&mut self) {
        if self.game_over() {
            return;
        }
        self.bot.playout()
    }

    pub fn get_visits(&self) -> f64 {
        self.bot
            .tree
            .get(&self.bot.root)
            .iter()
            .map(|tally| tally.visits.iter().map(|(_, (v, _))| v).sum::<u64>())
            .sum::<u64>() as f64
    }

    /**
     * Number of times we've tried and won by placing a penguin at `dst` on the current board
     */
    pub fn place_info(&self, dst: u8) -> MoveInfo {
        let (visits, rewards) = self
            .bot
            .tree
            .get(&self.bot.root)
            .map(|tally| tally.get_visit(htmf_bots::mctsbot::Move::Place(dst)))
            .unwrap_or((0, 0.0));
        MoveInfo { visits, rewards }
    }

    /**
     * Number of times we've tried and won by moving a penguin from `src` to `dst` on the current
     * board
     */
    pub fn move_info(&self, src: u8, dst: u8) -> MoveInfo {
        let (visits, rewards) = self
            .bot
            .tree
            .get(&self.bot.root)
            .map(|tally| tally.get_visit(htmf_bots::mctsbot::Move::Move((src, dst))))
            .unwrap_or((0, 0.0));
        MoveInfo { visits, rewards }
    }

    pub fn take_action(&mut self) -> Result<(), JsValue> {
        if self.game_over() {
            return Ok(());
        }
        while self.bot.root.state.active_player() == Some(self.bot.me) {
            let action = self.bot.take_action();
            let mut new_game = self.bot.root.state.clone();
            match new_game.apply_action(&action) {
                Ok(()) => {
                    self.bot.update(new_game);
                    Ok(())
                }
                Err(err) => Err(JsValue::from(err.message)),
            }?;
        }
        Ok(())
    }

    pub fn tree_size(&self) -> usize {
        self.bot.tree_size()
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

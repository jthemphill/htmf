use rand::prelude::*;
use wasm_bindgen::prelude::*;

use htmf::board::Player;
use htmf::game::GameState;
use htmf_bots::MCTSBot;

mod utils;

#[wasm_bindgen]
pub struct MoveInfo {
    visits: u32,
    rewards: f32,
}

#[wasm_bindgen]
impl MoveInfo {
    pub fn get_visits(&self) -> f32 {
        self.visits as f32
    }

    pub fn get_rewards(&self) -> f32 {
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
        self.bot.root.game.state.finished_drafting()
    }

    pub fn game_over(&self) -> bool {
        self.bot.root.game.state.game_over()
    }

    pub fn active_player(&self) -> Option<u8> {
        self.bot.root.game.state.active_player().map(|p| p.id as u8)
    }

    pub fn score(&self, player: usize) -> usize {
        self.bot.root.game.state.get_scores()[player]
    }

    pub fn turn(&self) -> usize {
        self.bot.root.game.state.turn
    }

    pub fn num_fish(&self, idx: u8) -> usize {
        self.bot.root.game.state.board.num_fish(idx)
    }

    pub fn penguins(&self, player: usize) -> Vec<u8> {
        self.bot.root.game.state.board.penguins[player]
            .into_iter()
            .collect()
    }

    pub fn claimed(&self, player: usize) -> Vec<u8> {
        self.bot.root.game.state.board.claimed[player]
            .into_iter()
            .collect()
    }

    pub fn draftable_cells(&self) -> Vec<u8> {
        self.bot.root.game.state.board.fish[0]
            .exclude(self.bot.root.game.state.board.all_claimed_cells())
            .into_iter()
            .collect()
    }

    pub fn possible_moves(&self, src: u8) -> Vec<u8> {
        self.bot
            .root
            .game
            .state
            .board
            .moves(src)
            .into_iter()
            .collect()
    }

    pub fn place_penguin(&mut self, dst: u8) -> Result<(), JsValue> {
        let mut new_game = self.bot.root.game.state.clone();
        match new_game.place_penguin(dst) {
            Ok(()) => {
                self.bot.update(&new_game);
                Ok(())
            }
            Err(err) => Err(JsValue::from(err.message)),
        }
    }

    pub fn move_penguin(&mut self, src: u8, dst: u8) -> Result<(), JsValue> {
        let mut new_game = self.bot.root.game.state.clone();
        match new_game.move_penguin(src, dst) {
            Ok(()) => {
                self.bot.update(&new_game);
                Ok(())
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
        if let Some(children) = self.bot.root.children.get() {
            children
                .iter()
                .map(|(_, child)| child.rewards_visits.get().1)
                .sum::<u32>() as f64
        } else {
            0.0
        }
    }

    /**
     * Number of times we've tried and won by placing a penguin at `dst` on the current board
     */
    pub fn place_info(&self, dst: u8) -> MoveInfo {
        self.info(htmf_bots::mctsbot::Move::Place(dst))
    }

    /**
     * Number of times we've tried and won by moving a penguin from `src` to `dst` on the current
     * board
     */
    pub fn move_info(&self, src: u8, dst: u8) -> MoveInfo {
        self.info(htmf_bots::mctsbot::Move::Move((src, dst)))
    }

    fn info(&self, game_move: htmf_bots::mctsbot::Move) -> MoveInfo {
        if let Some(children) = self.bot.root.children.get() {
            for (child_move, child) in children {
                if *child_move == game_move {
                    let (rewards, visits) = child.rewards_visits.get();
                    return MoveInfo { rewards, visits };
                }
            }
        }
        MoveInfo {
            visits: 0,
            rewards: 0.0,
        }
    }

    pub fn take_action(&mut self) -> Result<(), JsValue> {
        if self.game_over() {
            return Ok(());
        }
        while self.bot.root.game.state.active_player() == Some(self.bot.me) {
            let action = self.bot.take_action();
            let mut new_game = self.bot.root.game.state.clone();
            match new_game.apply_action(&action) {
                Ok(()) => {
                    self.bot.update(&new_game);
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

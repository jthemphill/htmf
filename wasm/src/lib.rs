use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

use wasm_bindgen::prelude::*;

use htmf::board::Player;
use htmf::game::GameState;
use htmf_bots::MCTSBot;

mod utils;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

#[cfg(feature = "parallel")]
pub use wasm_bindgen_rayon::init_thread_pool;

#[cfg(not(feature = "parallel"))]
#[wasm_bindgen(js_name = initThreadPool)]
/// This function only exists when you compile without multithreading support!
///
/// This function does nothing; it's just a stub function to match
/// `wasm_bindgen_rayon::init_thread_pool`, which exists when you do compile with
/// multithreading support.
pub fn init_thread_pool(_num_threads: usize) -> js_sys::Promise {
    js_sys::Promise::resolve(&JsValue::UNDEFINED)
}

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
    bot: Arc<RwLock<MCTSBot>>,
    total_playouts: AtomicUsize,
}

#[wasm_bindgen]
impl Game {
    pub fn new() -> Self {
        utils::set_panic_hook();
        Self {
            bot: Arc::from(RwLock::from(MCTSBot::new(
                GameState::new_two_player(&mut rand::rng()),
                Player { id: 1 },
            ))),
            total_playouts: AtomicUsize::from(0),
        }
    }

    pub fn is_drafting(&self) -> bool {
        !self.finished_drafting()
    }

    pub fn finished_drafting(&self) -> bool {
        let bot = self.bot.read().unwrap();
        bot.root.game.state.finished_drafting()
    }

    pub fn game_over(&self) -> bool {
        let bot = self.bot.read().unwrap();
        bot.root.game.state.game_over()
    }

    pub fn active_player(&self) -> Option<u8> {
        let bot = self.bot.read().unwrap();
        bot.root.game.state.active_player().map(|p| p.id as u8)
    }

    pub fn score(&self, player: usize) -> usize {
        let bot = self.bot.read().unwrap();
        bot.root.game.state.get_scores()[player]
    }

    pub fn turn(&self) -> usize {
        let bot = self.bot.read().unwrap();
        bot.root.game.state.turn
    }

    pub fn num_fish(&self, idx: u8) -> usize {
        let bot = self.bot.read().unwrap();
        bot.root.game.state.board.num_fish(idx)
    }

    pub fn penguins(&self, player: usize) -> Vec<u8> {
        let bot = self.bot.read().unwrap();
        bot.root.game.state.board.penguins[player]
            .into_iter()
            .collect()
    }

    pub fn claimed(&self, player: usize) -> Vec<u8> {
        let bot = self.bot.read().unwrap();
        bot.root.game.state.board.claimed[player]
            .into_iter()
            .collect()
    }

    pub fn draftable_cells(&self) -> Vec<u8> {
        let bot = self.bot.read().unwrap();
        bot.root.game.state.board.fish[0]
            .exclude(bot.root.game.state.board.all_claimed_cells())
            .into_iter()
            .collect()
    }

    pub fn possible_moves(&self, src: u8) -> Vec<u8> {
        let bot = self.bot.read().unwrap();
        bot.root.game.state.board.moves(src).into_iter().collect()
    }

    pub fn place_penguin(&mut self, dst: u8) -> Result<(), JsValue> {
        let mut bot = self.bot.write().unwrap();
        let mut new_game = bot.root.game.state.clone();
        match new_game.place_penguin(dst) {
            Ok(()) => {
                bot.update(&new_game);
                Ok(())
            }
            Err(err) => Err(JsValue::from(err.message)),
        }
    }

    pub fn move_penguin(&mut self, src: u8, dst: u8) -> Result<(), JsValue> {
        let mut bot = self.bot.write().unwrap();
        let mut new_game = bot.root.game.state.clone();
        match new_game.move_penguin(src, dst) {
            Ok(()) => {
                bot.update(&new_game);
                Ok(())
            }
            Err(err) => Err(JsValue::from(err.message)),
        }
    }

    pub fn playout(&self) {
        let bot = self.bot.read().unwrap();
        if bot.root.game.state.game_over() {
            return;
        }
        bot.playout();
        self.total_playouts.fetch_add(1, Ordering::Relaxed);
    }

    #[cfg(feature = "rayon")]
    pub fn playout_n_times(&self, n: usize) {
        if self.game_over() {
            return;
        }
        (0..n).into_par_iter().for_each(move |_| {
            let bot = self.bot.read().unwrap();
            bot.playout();
        });
        self.total_playouts.fetch_add(n, Ordering::Relaxed);
    }

    #[cfg(not(feature = "rayon"))]
    pub fn playout_n_times(&self, n: usize) {
        let bot = self.bot.read().unwrap();
        if bot.root.game.state.game_over() {
            return;
        }
        for _ in 0..n {
            bot.playout();
        }
        self.total_playouts.fetch_add(n, Ordering::Relaxed);
    }

    pub fn get_total_playouts(&self) -> usize {
        self.total_playouts.load(Ordering::Relaxed)
    }

    pub fn get_visits(&self) -> f64 {
        let bot = self.bot.read().unwrap();
        if let Some(children) = bot.root.children.get() {
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
        let bot = self.bot.read().unwrap();
        if let Some(children) = bot.root.children.get() {
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
        let mut bot = self.bot.write().unwrap();
        if bot.root.game.state.game_over() {
            return Ok(());
        }
        while bot.root.game.state.active_player() == Some(bot.me) {
            let action = bot.take_action();
            let mut new_game = bot.root.game.state.clone();
            match new_game.apply_action(&action) {
                Ok(()) => {
                    bot.update(&new_game);
                    Ok(())
                }
                Err(err) => Err(JsValue::from(err.message)),
            }?;
        }
        Ok(())
    }

    pub fn tree_size(&self) -> usize {
        let bot = self.bot.read().unwrap();
        bot.tree_size()
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

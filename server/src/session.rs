extern crate htmf;
extern crate htmf_bots;

use self::htmf_bots::*;
use htmf::board::Player;
use htmf::errors::IllegalMoveError;
use htmf::game::{Action, GameState};

#[derive(Clone)]
pub struct Session {
    pub game: GameState,
    bot: MCTSBot,
}

impl Session {
    pub fn new(game: GameState) -> Session {
        let bot = MCTSBot::new(&game, Player { id: 1 });
        Session { game: game, bot }
    }

    pub fn apply_action(&mut self, action: &Action) -> Result<(), IllegalMoveError> {
        let res = match action {
            &Action::Move(src, dst) => {
                self.game.move_penguin(src, dst)?;
                self.bot.update(&self.game);
                Ok(())
            }
            &Action::Place(cell_idx) => {
                self.game.place_penguin(cell_idx)?;
                self.bot.update(&self.game);
                Ok(())
            }
            &Action::Selection(_) => Ok(()),
            &Action::Setup(ref new_game_state) => {
                self.game = new_game_state.clone();
                self.bot.update(&self.game);
                Ok(())
            }
        };
        if res.is_err() {
            return res;
        }
        if let Some(active_player) = self.game.active_player() {
            if self.bot.me == active_player {
                let action = self.bot.take_action();
                self.apply_action(&action).unwrap();
            }
        }
        Ok(())
    }
}

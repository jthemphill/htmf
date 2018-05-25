extern crate htmf;
extern crate rand;
use htmf::board::{NUM_CELLS, Player};
use htmf::game::{Action, GameState};

#[derive(Clone)]
pub struct RandomBot {
    pub me: Player,
    game: GameState,
    rng: rand::XorShiftRng,
}

impl RandomBot {
    pub fn new(game: &GameState, me: Player) -> Self {
        RandomBot {
            game: game.clone(),
            me,
            rng: rand::weak_rng(),
        }
    }

    pub fn update(&mut self, game: &GameState) {
        self.game = game.clone();
    }

    pub fn take_action(&mut self) -> Action {
        if self.game.active_player() != Some(self.me) {
            panic!("{:?} was asked to move, but it is not their turn!", self.me);
        }
        if self.game.finished_drafting() {
            let src = rand::seq::sample_iter(
                &mut self.rng,
                self.game.board.penguins[self.me.id].iter(),
                1,
            ).unwrap()[0];
            let dst =
                rand::seq::sample_iter(&mut self.rng, self.game.board.moves(src), 1).unwrap()[0];
            Action::Move(src, dst)
        } else {
            let draftable_cells: Vec<usize> = (0..NUM_CELLS).into_iter()
                .filter(|&c| !self.game.board.is_claimed(c) && self.game.board.num_fish(c) == 1)
                .collect();
            Action::Place(
                rand::seq::sample_iter(
                    &mut self.rng,
                    draftable_cells,
                    1,
                ).unwrap()[0],
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_players_no_illegal_moves() {
        let mut game = GameState::new_two_player(&[0]);
        let mut bots = vec![
            RandomBot::new(&game, Player { id: 0 }),
            RandomBot::new(&game, Player { id: 1 }),
        ];
        while let Some(player) = game.active_player() {
            {
                let mut bot = &mut bots[player.id];
                let action = bot.take_action();
                game.apply_action(&action).unwrap();
            }
            for mut bot in &mut bots {
                bot.update(&game);
            }
        }
    }
}

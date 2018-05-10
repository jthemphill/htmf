extern crate rand;
extern crate rayon;

extern crate htmf;

use self::rayon::prelude::*;

use htmf::board::Player;
use htmf::game::{Action, GameState};

#[derive(Clone)]
pub struct MinimaxBot {
    pub me: Player,
    game: GameState,
    rng: rand::XorShiftRng,
    ply: i32,
}

impl MinimaxBot {
    pub fn new(game: &GameState, me: Player) -> Self {
        MinimaxBot {
            game: game.clone(),
            me,
            rng: rand::weak_rng(),
            ply: 2,
        }
    }

    pub fn new_with_ply(game: &GameState, me: Player, ply: i32) -> Self {
        MinimaxBot {
            game: game.clone(),
            me,
            rng: rand::weak_rng(),
            ply,
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
            let (_, (src, dst)) = Self::best_move(&self.game, &self.me, self.ply);
            Action::Move(src, dst)
        } else {
            Action::Place(rand::seq::sample_iter(
                &mut self.rng,
                self.game.board.cells.iter().enumerate()
                    .filter(|&(_, c)| c.claimed == None && c.fish == 1)
                    .map(|(idx, _)| idx),
                1,
            ).unwrap()[0])
        }
    }

    fn best_move(
        game: &GameState,
        p: &Player,
        ply: i32,
    ) -> (Vec<usize>, (usize, usize)) {
        Self::all_moves(game, p).into_par_iter()
            .map(|mv| (Self::score_move(game, &mv, ply), mv))
            .max_by_key(|&(ref scores, _)| Self::negamax_score(scores, p))
            .unwrap()
    }

    fn score_move(
        game: &GameState,
        mv: &(usize, usize),
        ply: i32,
    ) -> Vec<usize> {
        if ply <= 0 {
            return game.scores.iter().cloned().collect();
        }
        let game = {
            let mut game = game.clone();
            game.move_penguin(mv.0, mv.1).unwrap();
            game
        };
        if let Some(new_active_player) = game.active_player() {
            let (scores, _) = Self::best_move(&game, &new_active_player, ply - 1);
            scores
        } else {
            game.scores.iter().cloned().collect()
        }
    }

    fn negamax_score(scores: &Vec<usize>, p: &Player) -> i32 {
        let my_score = scores[p.id];
        let best_other_score = scores.iter().enumerate()
            .filter(|&(i, _)| i != p.id)
            .map(|(_, &score)| score)
            .max()
            .unwrap_or(0);
        my_score as i32 - best_other_score as i32
    }

    fn all_moves(game: &GameState, p: &Player) -> Vec<(usize, usize)> {
        game.board.penguins[p.id].into_iter().flat_map(
            |src| game.board.moves(src).into_iter()
                .map(move |dst| (src, dst))
        ).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_players_no_illegal_moves() {
        let mut game = GameState::new_two_player(&[0]);
        let mut bots = vec![
            MinimaxBot::new_with_ply(&game, Player{id:0}, 0),
            MinimaxBot::new_with_ply(&game, Player{id:1}, 0),
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

extern crate rand;
extern crate rayon;

extern crate htmf;

use self::rayon::prelude::*;
use rand::prelude::*;

use htmf::board::Player;
use htmf::game::{Action, GameState};

type Move = (u8, u8);

#[derive(Clone)]
pub struct MinimaxBot {
    pub me: Player,
    game: GameState,
    rng: ThreadRng,
    ply: i32,
}

impl MinimaxBot {
    pub fn new(game: &GameState, me: Player) -> Self {
        MinimaxBot {
            game: game.clone(),
            me,
            rng: thread_rng(),
            ply: 2,
        }
    }

    pub fn new_with_ply(game: &GameState, me: Player, ply: i32) -> Self {
        MinimaxBot {
            game: game.clone(),
            me,
            rng: thread_rng(),
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
            // Cells with one fish and nobody claiming them
            let mut draftable_cells = self.game.board.fish[0].clone();
            draftable_cells.exclude(&self.game.board.all_claimed_cells());
            Action::Place(draftable_cells.iter().choose(&mut self.rng).unwrap())
        }
    }

    fn best_move(game: &GameState, p: &Player, ply: i32) -> (Vec<usize>, Move) {
        Self::all_moves(game, *p)
            .into_par_iter()
            .map(|mv| (Self::score_move(game, &mv, ply), mv))
            .max_by_key(|&(ref scores, _)| Self::negamax_score(scores, p))
            .unwrap()
    }

    fn score_move(game: &GameState, mv: &Move, ply: i32) -> Vec<usize> {
        if ply <= 0 {
            return (0..game.nplayers)
                .into_iter()
                .map(|i| game.board.get_score(Player { id: i }))
                .collect();
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
            (0..game.nplayers)
                .into_iter()
                .map(|i| game.board.get_score(Player { id: i }))
                .collect()
        }
    }

    fn negamax_score(scores: &[usize], p: &Player) -> i32 {
        let my_score = scores[p.id];
        let best_other_score = scores
            .iter()
            .enumerate()
            .filter(|&(i, _)| i != p.id)
            .map(|(_, &score)| score)
            .max()
            .unwrap_or(0);
        my_score as i32 - best_other_score as i32
    }

    fn all_moves(game: &GameState, p: Player) -> Vec<Move> {
        game.board.penguins[p.id]
            .into_iter()
            .flat_map(|src| {
                let move_vec: Vec<Move> = game
                    .board
                    .moves(src)
                    .into_iter()
                    .map(|dst| (src, dst))
                    .collect();
                move_vec.into_iter()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_players_no_illegal_moves() {
        let mut game = GameState::new_two_player([0; 32]);
        let mut bots = vec![
            MinimaxBot::new_with_ply(&game, Player { id: 0 }, 0),
            MinimaxBot::new_with_ply(&game, Player { id: 1 }, 0),
        ];
        while let Some(player) = game.active_player() {
            let bot = &mut bots[player.id];
            let action = bot.take_action();
            game.apply_action(&action).unwrap();
            for bot in &mut bots {
                bot.update(&game);
            }
        }
    }
}

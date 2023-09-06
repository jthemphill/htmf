use rand::prelude::*;
use std::fmt;

use arrayvec::ArrayVec;

use crate::board::{Board, Player};
use crate::errors::IllegalMoveError;
use crate::json::GameStateJSON;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct GameState {
    pub nplayers: usize,
    pub turn: usize,
    pub board: Board,
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        GameStateJSON::from(self).fmt(f)
    }
}

impl GameState {
    // Predicates

    pub fn finished_drafting(&self) -> bool {
        let limit = match self.nplayers {
            2 => 8,
            3 => 9,
            4 => 8,
            _ => panic!("Illegal number of players"),
        };
        self.turn >= limit
    }

    pub fn game_over(&self) -> bool {
        self.finished_drafting() && self.board.penguins.iter().all(|&p| p.is_empty())
    }

    pub fn active_player(&self) -> Option<Player> {
        if self.game_over() {
            return None;
        }
        if self.finished_drafting() {
            let mut p = self.turn % self.nplayers;
            while self.board.penguins[p].is_empty() {
                p += 1;
                p %= self.nplayers;
            }
            Some(Player { id: p })
        } else if self.turn / self.nplayers % 2 == 1 {
            let mut turn = -(self.turn as isize) - 1;
            turn %= self.nplayers as isize;
            if turn < 0 {
                turn += self.nplayers as isize;
            }
            Some(Player { id: turn as usize })
        } else {
            Some(Player {
                id: self.turn % self.nplayers,
            })
        }
    }

    pub fn get_scores(&self) -> ArrayVec<usize, 4> {
        (0..self.nplayers)
            .map(|i| self.board.get_score(Player { id: i }))
            .collect()
    }

    pub fn is_legal_place(&self, c: u8) -> bool {
        !self.board.is_claimed(c)
    }

    pub fn is_legal_move(&self, src: u8, dst: u8) -> bool {
        self.board
            .is_legal_move(self.active_player().unwrap(), src, dst)
    }

    // Actions

    pub fn apply_action(&mut self, action: &Action) -> Result<(), IllegalMoveError> {
        match *action {
            Action::Move(src, dst) => self.move_penguin(src, dst),
            Action::Place(cell_idx) => self.place_penguin(cell_idx),
            _ => unimplemented!(),
        }
    }

    pub fn new_two_player<R: Rng + ?Sized>(rng: &mut R) -> GameState {
        let nplayers = 2;
        GameState {
            nplayers,
            turn: 0,
            board: Board::new(rng),
        }
    }

    pub fn place_penguin(&mut self, c: u8) -> Result<(), IllegalMoveError> {
        if self.finished_drafting() {
            return Err(IllegalMoveError::new(
                self.active_player().unwrap(),
                "Drafting phase is over".to_owned(),
            ));
        }
        if self.board.num_fish(c) != 1 {
            return Err(IllegalMoveError::new(
                self.active_player().unwrap(),
                "You must place on a cell containing one fish!".to_owned(),
            ));
        }
        let active_player = self.active_player().unwrap();
        self.board.claim_cell(active_player, c)?;
        self.turn += 1;
        Ok(())
    }

    pub fn move_penguin(&mut self, src: u8, dst: u8) -> Result<(), IllegalMoveError> {
        if !self.finished_drafting() {
            return Err(IllegalMoveError::new(
                self.active_player().unwrap(),
                "Drafting phase is not over".to_owned(),
            ));
        }
        let active_player = self.active_player().unwrap();
        self.board.move_penguin(active_player, src, dst)?;
        let mut needs_prune = self.board.is_cut_cell(src);
        while needs_prune {
            needs_prune = self.board.prune();
        }
        self.board.reap();
        self.turn += 1;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum Action {
    Move(u8, u8),
    Place(u8),
    Selection(u8),
    Setup(GameState),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draft_turn_sequence_is_good() {
        let g = GameState::new_two_player::<StdRng>(&mut SeedableRng::seed_from_u64(0));

        let actives: Vec<usize> = (0..8)
            .scan(g, |g, _| {
                let p = g.active_player().unwrap();
                g.turn += 1;
                Some(p.id)
            })
            .collect();
        let expected = vec![0, 1, 1, 0, 0, 1, 1, 0];
        assert_eq!(actives, expected);
    }

    #[test]
    fn game_not_over_after_draft_is_over() {
        let mut g = GameState::new_two_player::<StdRng>(&mut SeedableRng::seed_from_u64(0));

        for _ in 0..8 {
            assert!(!g.finished_drafting());
            assert!(!g.game_over());
            assert!(g.active_player().is_some());
            let one_fish_cell = g.board.fish[0]
                .into_iter()
                .filter(|&i| !g.board.is_claimed(i))
                .next()
                .unwrap();
            g.place_penguin(one_fish_cell).unwrap();
        }

        assert!(g.finished_drafting());
        assert!(!g.game_over());
        assert!(g.active_player().is_some());
    }
}

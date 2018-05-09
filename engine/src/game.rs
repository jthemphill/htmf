use board::{Board, Player};
use errors::IllegalMoveError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameState {
    pub nplayers: usize,
    pub scores: Vec<usize>,
    pub turn: usize,
    pub board: Board,
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
        self.finished_drafting() && self.board.penguins.iter()
            .find(|&p| !p.is_empty())
            .is_none()
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
            Some(Player{id: p})
        } else if self.turn / self.nplayers % 2 == 1 {
            let mut turn = -(self.turn as isize) - 1;
            turn %= self.nplayers as isize;
            if turn < 0 {
                turn += self.nplayers as isize;
            }
            Some(Player{
                id: turn as usize,
            })
        } else {
            Some(Player{
                id: self.turn % self.nplayers,
            })
        }
    }

    pub fn is_legal_place(&self, c: usize) -> bool {
        self.board.cells[c].claimed == None
    }

    pub fn is_legal_move(&self, src: usize, dst: usize) -> bool {
        self.board.is_legal_move(self.active_player().unwrap(), src, dst)
    }

    // Actions

    pub fn apply_action(&mut self, action: &Action) -> Result<(), IllegalMoveError> {
        match action {
            &Action::Move(src, dst) => self.move_penguin(src, dst),
            &Action::Place(cell_idx) => self.place_penguin(cell_idx),
            _ => unimplemented!(),
        }
    }

    pub fn new_two_player(seed: &[usize]) -> GameState {
        GameState {
            nplayers: 2,
            turn: 0,
            scores: vec![0, 0],
            board: Board::new(seed),
        }
    }

    pub fn place_penguin(
        &mut self,
        c: usize,
    ) -> Result<(), IllegalMoveError> {
        if self.finished_drafting() {
            return Err(IllegalMoveError::new(
                self.active_player().unwrap(),
                "Drafting phase is over".to_owned(),
            ));
        }
        if self.board.cells[c].fish != 1 {
            return Err(IllegalMoveError::new(
                self.active_player().unwrap(),
                "You must place on a cell containing one fish!".to_owned(),
            ));
        }
        let active_player = self.active_player().unwrap();
        try!(self.board.claim_cell(active_player, c));
        self.scores[active_player.id] += 1;
        self.turn += 1;
        Ok(())
    }

    pub fn move_penguin(
        &mut self,
        src: usize,
        dst: usize,
    ) -> Result<(), IllegalMoveError> {
        if !self.finished_drafting() {
            return Err(
                IllegalMoveError::new(
                    self.active_player().unwrap(),
                    "Drafting phase is not over".to_owned(),
                ));
        }
        let active_player = self.active_player().unwrap();
        try!(self.board.move_penguin(active_player, src, dst));
        let mut has_pruned = self.board.prune();
        while has_pruned {
            has_pruned = self.board.prune();
        }
        self.board.reap();
        self.scores[active_player.id] = self.board.get_score(&active_player);
        self.turn += 1;
        Ok(())
    }
}

#[derive(Debug)]
pub enum Action {
    Move(usize, usize),
    Place(usize),
    Selection(usize),
    Setup(GameState),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draft_turn_sequence_is_good() {
        let g = GameState::new_two_player(&[0]);

        let actives: Vec<usize> = (0..8).scan(
            g,
            |g, _| {
                let p = g.active_player().unwrap();
                g.turn += 1;
                Some(p.id)
            }
        ).collect();
        let expected = vec![0, 1, 1, 0, 0, 1, 1, 0];
        assert_eq!(actives, expected);
    }
}

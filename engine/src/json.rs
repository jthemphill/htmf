use std::fmt;

use serde_derive::{Deserialize, Serialize};
use serde_json;

use crate::board::Board;
use crate::cellset::CellSet;
use crate::game::GameState;
use crate::NUM_CELLS;

const NUM_PLAYERS: usize = 2;

#[derive(Debug, Serialize, Deserialize)]
pub struct GameStateJSON {
    pub last_move_valid: bool,
    pub mode_type: GameModeType,
    pub nplayers: usize,
    pub active_player: Option<usize>,
    pub scores: Vec<usize>,
    pub turn: usize,
    pub board: BoardJSON,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BoardJSON {
    pub fish: Vec<usize>,
    pub penguins: Vec<Vec<u8>>,
    pub claimed: Vec<Vec<u8>>,
    pub possible_moves: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GameModeType {
    #[serde(rename = "drafting")]
    Drafting,
    #[serde(rename = "playing")]
    Playing,
}

impl fmt::Display for GameStateJSON {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        serde_json::to_string(self).unwrap().fmt(f)
    }
}

impl<'a> From<&'a GameState> for GameStateJSON {
    fn from(state: &'a GameState) -> Self {
        let mode_type = if state.finished_drafting() {
            GameModeType::Playing
        } else {
            GameModeType::Drafting
        };

        GameStateJSON {
            last_move_valid: true,
            mode_type,
            nplayers: NUM_PLAYERS,
            active_player: state.active_player().map(|p| p.id),
            scores: state.get_scores().to_vec(),
            turn: state.turn,
            board: BoardJSON::from(&state.board),
        }
    }
}

impl<'a> From<&'a Board> for GameStateJSON {
    fn from(b: &'a Board) -> Self {
        GameStateJSON {
            last_move_valid: true,
            mode_type: GameModeType::Drafting,
            scores: vec![0, 0],
            nplayers: NUM_PLAYERS,
            active_player: Some(0),
            turn: 0,
            board: BoardJSON::from(b),
        }
    }
}

impl<'a> From<&'a GameStateJSON> for GameState {
    fn from(json: &'a GameStateJSON) -> Self {
        GameState {
            turn: json.turn,
            board: Board::from(&json.board),
        }
    }
}

impl<'a> From<&'a GameStateJSON> for String {
    fn from(state: &'a GameStateJSON) -> String {
        serde_json::to_string(state).unwrap()
    }
}

impl<'a> From<&'a Board> for BoardJSON {
    fn from(b: &'a Board) -> Self {
        BoardJSON {
            fish: (0..NUM_CELLS as u8).map(|c| b.num_fish(c)).collect(),
            claimed: b
                .claimed
                .iter()
                .map(|cells| cells.into_iter().collect())
                .collect(),
            penguins: b
                .penguins
                .iter()
                .map(|cells| cells.into_iter().collect())
                .collect(),
            possible_moves: None,
        }
    }
}

impl<'a> From<&'a BoardJSON> for Board {
    fn from(b: &'a BoardJSON) -> Self {
        let mut fish: [CellSet; 3] = [CellSet::new(); 3];
        for num_fish in 1..=3 {
            fish[num_fish] = b
                .fish
                .iter()
                .enumerate()
                .filter(|&(_, fish)| *fish == num_fish)
                .map(|(i, _)| i as u8)
                .collect()
        }

        let mut penguins: [CellSet; 2] = [CellSet::new(); 2];
        for player in 0..2 {
            if let Some(player_penguins) = b.penguins.get(player) {
                for &p in player_penguins {
                    penguins[player] = penguins[player].insert(p as u8);
                }
            }
        }

        let mut claimed: [CellSet; 2] = [CellSet::new(); 2];
        for player in 0..2 {
            claimed[player] = b.claimed[player].iter().cloned().collect();
        }

        Board {
            fish,
            penguins,
            claimed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::prelude::*;

    #[test]
    fn initial_state() {
        let game = GameState::new_two_player::<StdRng>(&mut SeedableRng::seed_from_u64(0));
        let game_json = GameStateJSON::from(&game);
        let game_again = GameState::from(&game_json);
        assert_eq!(game, game_again);
    }

    #[test]
    fn after_claiming() {
        let mut game = GameState::new_two_player::<StdRng>(&mut SeedableRng::seed_from_u64(0));
        let eligible_place = (0..NUM_CELLS as u8)
            .find(|&cell| game.board.num_fish(cell) == 1)
            .unwrap();
        game.place_penguin(eligible_place).unwrap();
        let game_json = GameStateJSON::from(&game);
        let game_again = GameState::from(&game_json);
        assert_eq!(game, game_again);
    }
}

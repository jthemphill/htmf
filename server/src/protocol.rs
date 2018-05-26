#![allow(dead_code)]

extern crate bit_set;
extern crate serde;
extern crate serde_json;

extern crate htmf;

use self::bit_set::BitSet;
use htmf::board::{Board, NUM_CELLS};
use htmf::game::{Action, GameState};

/// Module for all input and output.
/// 1. We receive ActionJSON from the client
/// 2. We convert it into an Action enum, which we use internally.
/// 3. We send GameStateJSON back to the client.

pub fn action_from_str(action_str: &str) -> Option<Action> {
    let action = match serde_json::from_str(action_str) {
        Ok(action) => action,
        Err(e) => {
            println!("Error {}: {} is not a valid action", e, action_str);
            return None;
        }
    };
    if let Some(action) = get_action(action) {
        println!("Action: {}", action_str);
        Some(action)
    } else {
        println!("Invalid action: {}", action_str);
        None
    }
}

#[derive(Serialize, Deserialize)]
struct ActionJSON {
    pub action_type: ActionType,
    pub data: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
enum ActionType {
    #[serde(rename = "move")]
    Move,
    #[serde(rename = "place")]
    Place,
    #[serde(rename = "selection")]
    Selection,
    #[serde(rename = "setup")]
    Setup,
}

#[derive(Serialize, Deserialize)]
struct ActionSelectionJSON {
    hex: u64,
}

#[derive(Serialize, Deserialize)]
struct ActionPlaceJSON {
    hex: u64,
}

#[derive(Serialize, Deserialize)]
struct ActionMoveJSON {
    src: u64,
    dst: u64,
}

#[derive(Serialize, Deserialize)]
struct ActionSetupJSON {
    state: GameStateJSON,
}

fn get_action(action_json: ActionJSON) -> Option<Action> {
    match action_json.action_type {
        ActionType::Selection => {
            let selection_data: ActionSelectionJSON =
                serde_json::from_value(action_json.data).unwrap_or(None)?;
            Some(Action::Selection(selection_data.hex as usize))
        }
        ActionType::Move => {
            let move_data: ActionMoveJSON =
                serde_json::from_value(action_json.data).unwrap_or(None)?;
            Some(Action::Move(move_data.src as usize, move_data.dst as usize))
        }
        ActionType::Place => {
            let place_data: ActionPlaceJSON =
                serde_json::from_value(action_json.data).unwrap_or(None)?;
            Some(Action::Place(place_data.hex as usize))
        }
        ActionType::Setup => {
            println!("Setting up?");
            let game_state_data: ActionSetupJSON = match serde_json::from_value(action_json.data) {
                Ok(data) => Some(data),
                Err(e) => {
                    println!("Error setting state: {}", e);
                    None
                }
            }?;
            let setup_action = Action::Setup(game_state_data.state.to_game());
            Some(setup_action)
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct GameStateJSON {
    pub last_move_valid: bool,
    pub mode_type: GameModeType,
    pub nplayers: usize,
    pub active_player: Option<usize>,
    pub scores: Vec<usize>,
    pub turn: usize,
    pub board: BoardJSON,
}

impl GameStateJSON {
    pub fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn from_game(state: &GameState) -> Self {
        let mode_type = if state.finished_drafting() {
            GameModeType::Playing
        } else {
            GameModeType::Drafting
        };

        GameStateJSON {
            last_move_valid: true,
            mode_type,
            nplayers: state.nplayers,
            active_player: match state.active_player() {
                Some(p) => Some(p.id),
                _ => None,
            },
            scores: state.scores.to_vec(),
            turn: state.turn,
            board: BoardJSON::from_board(&state.board),
        }
    }

    pub fn to_game(&self) -> GameState {
        GameState {
            nplayers: self.nplayers,
            turn: self.turn,
            scores: self.scores.iter().cloned().collect(),
            board: self.board.to_native(),
        }
    }

    pub fn from_board(b: &Board) -> Self {
        GameStateJSON {
            last_move_valid: true,
            mode_type: GameModeType::Drafting,
            scores: vec![0, 0],
            nplayers: 2,
            active_player: Some(0),
            turn: 0,
            board: BoardJSON::from_board(b),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum GameModeType {
    #[serde(rename = "drafting")]
    Drafting,
    #[serde(rename = "playing")]
    Playing,
}

#[derive(Serialize, Deserialize)]
pub struct BoardJSON {
    pub fish: Vec<usize>,
    pub penguins: Vec<Vec<usize>>,
    pub claimed: Vec<Vec<i32>>,
    pub possible_moves: Vec<usize>,
}

impl BoardJSON {
    pub fn from_board(b: &Board) -> Self {
        BoardJSON {
            fish: (0..NUM_CELLS).map(|c| b.num_fish(c)).collect(),
            claimed: b.claimed.iter().map(|cells| cells.into_iter().map(|c| c as i32).collect()).collect(),
            penguins: b.penguins.iter().map(|cells| cells.into_iter().collect()).collect(),
            possible_moves: vec![],
        }
    }

    fn to_native(&self) -> Board {
        let fish = (1..=3).into_iter()
            .map(|num_fish|
                self.fish.iter()
                    .enumerate()
                    .filter(|&(_, fish)| *fish == num_fish)
                    .map(|(i, _)| i)
                    .collect()
            ).collect();
        let penguins = (0..=4)
            .into_iter()
            .map(|player| {
                let mut penguin_set = BitSet::new();
                if let Some(player_penguins) = self.penguins.get(player) {
                    for &p in player_penguins {
                        penguin_set.insert(p);
                    }
                }
                penguin_set
            })
            .collect();
        let claimed = self.claimed.iter()
            .map(|cells| cells.iter().map(|&c| c as usize).collect())
            .collect();
        Board { fish, penguins, claimed }
    }

    pub fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

pub fn init_with_board(board: &Board) -> String {
    let state = GameStateJSON::from_board(board);

    serde_json::to_string(&state).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_state() {
        let game = GameState::new_two_player(&[0]);
        let game_json = GameStateJSON::from_game(&game);
        let game_again = game_json.to_game();
        assert_eq!(game, game_again);
    }

    #[test]
    fn after_claiming() {
        let mut game = GameState::new_two_player(&[0]);
        let eligible_place = (0..NUM_CELLS)
            .into_iter()
            .filter(|&cell| game.board.num_fish(cell) == 1)
            .next()
            .unwrap();
        game.place_penguin(eligible_place).unwrap();
        let game_json = GameStateJSON::from_game(&game);
        let game_again = game_json.to_game();
        assert_eq!(game, game_again);
    }
}

use serde_json;

use htmf::game::{Action, GameState};
use htmf::json::GameStateJSON;

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
    hex: u8,
}

#[derive(Serialize, Deserialize)]
struct ActionPlaceJSON {
    hex: u8,
}

#[derive(Serialize, Deserialize)]
struct ActionMoveJSON {
    src: u8,
    dst: u8,
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
            Some(Action::Selection(selection_data.hex))
        }
        ActionType::Move => {
            let move_data: ActionMoveJSON =
                serde_json::from_value(action_json.data).unwrap_or(None)?;
            Some(Action::Move(move_data.src, move_data.dst))
        }
        ActionType::Place => {
            let place_data: ActionPlaceJSON =
                serde_json::from_value(action_json.data).unwrap_or(None)?;
            Some(Action::Place(place_data.hex))
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
            let setup_action = Action::Setup(GameState::from(&game_state_data.state));
            Some(setup_action)
        }
    }
}

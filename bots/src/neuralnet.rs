use htmf::NUM_CELLS;
use tract_onnx::prelude::*;

const NUM_FEATURES: usize = 8 * NUM_CELLS;

type Model = SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>;

/// Neural network wrapper for policy and value prediction
pub struct NeuralNet {
    drafting_model: Model,
    movement_model: Model,
}

/// Output from neural network inference
pub struct NeuralNetOutput {
    /// Policy logits (60 for drafting, 3600 for movement)
    pub policy_logits: Vec<f32>,
    /// Value estimate (win probability for current player)
    pub value: f32,
}

impl NeuralNet {
    /// Load ONNX models from the given paths
    pub fn load(drafting_path: &str, movement_path: &str) -> TractResult<Self> {
        let drafting_model = tract_onnx::onnx()
            .model_for_path(drafting_path)?
            .with_input_fact(0, f32::fact([1, NUM_FEATURES]).into())?
            .into_optimized()?
            .into_runnable()?;

        let movement_model = tract_onnx::onnx()
            .model_for_path(movement_path)?
            .with_input_fact(0, f32::fact([1, NUM_FEATURES]).into())?
            .into_optimized()?
            .into_runnable()?;

        Ok(Self {
            drafting_model,
            movement_model,
        })
    }

    /// Run inference on the given game state
    pub fn predict(
        &self,
        game: &htmf::game::GameState,
        current_player: usize,
    ) -> TractResult<NeuralNetOutput> {
        let features = extract_features(game, current_player);
        let is_drafting = !game.finished_drafting();

        let input: Tensor = tract_ndarray::Array2::from_shape_vec((1, NUM_FEATURES), features)?
            .into();

        let model = if is_drafting {
            &self.drafting_model
        } else {
            &self.movement_model
        };

        let outputs = model.run(tvec!(input.into()))?;

        // Output 0: policy logits, Output 1: value
        let policy_logits: Vec<f32> = outputs[0]
            .to_array_view::<f32>()?
            .iter()
            .copied()
            .collect();
        let value: f32 = outputs[1].to_array_view::<f32>()?[[0, 0]];

        Ok(NeuralNetOutput {
            policy_logits,
            value,
        })
    }
}

/// Extract board features for neural network input
/// Layout: 8 channels x 60 cells = 480 values
fn extract_features(game: &htmf::game::GameState, current_player: usize) -> Vec<f32> {
    let opponent = 1 - current_player;
    let is_drafting = !game.finished_drafting();

    let mut features = vec![0.0f32; NUM_FEATURES];

    // Channels 0-2: fish counts
    for cell in 0..NUM_CELLS as u8 {
        if game.board.fish[0].contains(cell) {
            features[cell as usize] = 1.0;
        }
        if game.board.fish[1].contains(cell) {
            features[NUM_CELLS + cell as usize] = 1.0;
        }
        if game.board.fish[2].contains(cell) {
            features[2 * NUM_CELLS + cell as usize] = 1.0;
        }
    }

    // Channel 3: current player's penguins
    for cell in game.board.penguins[current_player].into_iter() {
        features[3 * NUM_CELLS + cell as usize] = 1.0;
    }

    // Channel 4: opponent's penguins
    for cell in game.board.penguins[opponent].into_iter() {
        features[4 * NUM_CELLS + cell as usize] = 1.0;
    }

    // Channel 5: current player's claimed cells
    for cell in game.board.claimed[current_player].into_iter() {
        features[5 * NUM_CELLS + cell as usize] = 1.0;
    }

    // Channel 6: opponent's claimed cells
    for cell in game.board.claimed[opponent].into_iter() {
        features[6 * NUM_CELLS + cell as usize] = 1.0;
    }

    // Channel 7: drafting phase indicator
    if is_drafting {
        for i in 0..NUM_CELLS {
            features[7 * NUM_CELLS + i] = 1.0;
        }
    }

    features
}

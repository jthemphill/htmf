#!/usr/bin/env python3
"""
Create blank ONNX models that output uniform policy and neutral value.

These models are useful for debugging:
- Policy: All zeros → softmax gives 1/n for each legal move (uniform)
- Value: Always 0 (tanh) → converted to 0.5 in Rust (neutral)

With these models, the AlphaZero bot should explore uniformly but still
use the PUCT formula. Comparing this to pure MCTS helps isolate bugs.

IMPORTANT: Even with these "blank" models, AlphaZero will NOT behave
identically to pure MCTS because:

1. PUCT vs UCB1 selection:
   - MCTS UCB1: unvisited nodes get INFINITY (always explore first)
   - AlphaZero PUCT: unvisited nodes get Q=0.5 + exploration term (finite)

2. Leaf evaluation:
   - MCTS: Random rollout to game end → actual win/loss/draw
   - Blank model: Always returns value=0.5 → Q-values stay at 0.5

3. This means with blank models:
   - All nodes will have Q ≈ 0.5 (no learning of which moves are good)
   - Selection driven purely by exploration term: C_PUCT * P * sqrt(N) / (1+n)
   - With uniform priors, exploration favors less-visited nodes

The UniformPriorRollout mode in Rust is a better debugging tool because
it uses random rollouts (like MCTS) but with PUCT selection.
"""

from pathlib import Path

import torch
import torch.nn as nn

# Constants matching the Rust/training code
NUM_CELLS = 60
NUM_CHANNELS = 8
NUM_FEATURES = NUM_CHANNELS * NUM_CELLS  # 480
MOVEMENT_POLICY_SIZE = 4 * 6 * 7  # 168 (penguins × directions × distances)

ARTIFACTS_DIR = Path("./artifacts")


class BlankHTMFNet(nn.Module):
    """
    Blank neural network that outputs uniform policy and neutral value.

    The model ignores the input and always outputs:
    - Policy: all zeros (softmax → uniform distribution)
    - Value: 0 (tanh → neutral, converts to 0.5 in Rust)
    """

    def __init__(self, policy_size: int):
        super().__init__()
        self.policy_size = policy_size
        # These parameters exist to give ONNX export something to work with,
        # but we override forward() to ignore them
        self.dummy_param = nn.Parameter(torch.zeros(1))

    def forward(self, x: torch.Tensor) -> tuple[torch.Tensor, torch.Tensor]:
        batch_size = x.shape[0]
        # Always output zeros for policy (softmax will make this uniform)
        policy = torch.zeros(batch_size, self.policy_size)
        # Always output zero for value (tanh=0 → neutral → 0.5 in Rust)
        value = torch.zeros(batch_size, 1)
        return policy, value


def export_to_onnx(model: nn.Module, path: Path):
    """Export model to ONNX format."""
    model.eval()
    dummy_input = torch.zeros(1, NUM_FEATURES)

    torch.onnx.export(
        model,
        dummy_input,
        path,
        input_names=["features"],
        output_names=["policy", "value"],
        dynamic_axes={
            "features": {0: "batch_size"},
            "policy": {0: "batch_size"},
            "value": {0: "batch_size"},
        },
        opset_version=17,
        dynamo=False,
    )


def main():
    ARTIFACTS_DIR.mkdir(parents=True, exist_ok=True)

    # Create blank drafting model
    drafting_model = BlankHTMFNet(NUM_CELLS)
    drafting_path = ARTIFACTS_DIR / "blank_model_drafting.onnx"
    print(f"Creating blank drafting model: {drafting_path}")
    export_to_onnx(drafting_model, drafting_path)

    # Create blank movement model
    movement_model = BlankHTMFNet(MOVEMENT_POLICY_SIZE)
    movement_path = ARTIFACTS_DIR / "blank_model_movement.onnx"
    print(f"Creating blank movement model: {movement_path}")
    export_to_onnx(movement_model, movement_path)

    print()
    print("Blank models created!")
    print()
    print("To use them, copy to the standard names:")
    print("  cp artifacts/blank_model_drafting.onnx artifacts/model_drafting.onnx")
    print("  cp artifacts/blank_model_movement.onnx artifacts/model_movement.onnx")
    print()
    print("Or modify nn_vs_mcts.rs to load them from the blank paths.")
    print()
    print("NOTE: These blank models output uniform policy (all zeros -> 1/n after softmax)")
    print("and neutral value (0 tanh -> 0.5 probability).")
    print()
    print("The PUCT mode uses random rollouts for leaf evaluation, so the value output")
    print("from these models is NOT used. Only the policy priors are used to guide")
    print("which moves to explore first.")


if __name__ == "__main__":
    main()

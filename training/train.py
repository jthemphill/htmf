#!/usr/bin/env python3
"""
Train a neural network for HTMF using PyTorch.

The network has two heads:
- Policy head: probability distribution over moves
  - Drafting: 60 values (one per cell)
  - Movement: 168 values (4 penguins × 6 directions × 7 distances)
- Value head: predicted win probability for current player

Usage:
    uv run train.py [--epochs N] [--lr RATE] [--batch-size B]
"""

import argparse
import json
from pathlib import Path

import numpy as np
import torch
import torch.nn as nn
import torch.nn.functional as F
from torch.utils.data import DataLoader, Dataset

# Constants matching the Rust code
NUM_CELLS = 60
NUM_CHANNELS = 8
NUM_FEATURES = NUM_CHANNELS * NUM_CELLS  # 480
NUM_PENGUINS = 4
NUM_DIRECTIONS = 6
MAX_DISTANCE = 7
MOVEMENT_POLICY_SIZE = NUM_PENGUINS * NUM_DIRECTIONS * MAX_DISTANCE  # 168

# Grid dimensions for Conv2D (8 rows, alternating 7/8 columns -> embed in 8x8)
NUM_ROWS = 8
NUM_COLS = 8
GRID_SIZE = NUM_ROWS * NUM_COLS  # 64

# Mapping from flat 60-cell index to 8x8 grid position
# Even rows (0,2,4,6) have 7 cells, odd rows (1,3,5,7) have 8 cells
def _build_cell_to_grid():
    """Build mapping from 60-cell index to (row, col) in 8x8 grid."""
    cell_to_grid = []
    cell_idx = 0
    for row in range(NUM_ROWS):
        row_len = 7 if row % 2 == 0 else 8
        for col in range(row_len):
            cell_to_grid.append((row, col))
            cell_idx += 1
    return cell_to_grid

CELL_TO_GRID = _build_cell_to_grid()

# Precompute valid cell mask for the 8x8 grid (60 valid, 4 invalid)
def _build_valid_mask():
    """Build a mask of valid cells in the 8x8 grid."""
    mask = torch.zeros(NUM_ROWS, NUM_COLS)
    for row, col in CELL_TO_GRID:
        mask[row, col] = 1.0
    return mask

VALID_CELL_MASK = _build_valid_mask()

ARTIFACTS_DIR = Path("./artifacts")
TRAINING_DATA = ARTIFACTS_DIR / "training_data.jsonl"
MODEL_CHECKPOINT = ARTIFACTS_DIR / "model_final.pt"
ONNX_DRAFTING = ARTIFACTS_DIR / "model_drafting.onnx"
ONNX_MOVEMENT = ARTIFACTS_DIR / "model_movement.onnx"


class HTMFDataset(Dataset):
    """Dataset for HTMF training samples."""

    def __init__(self, data_path: Path):
        self.drafting_samples: list[dict] = []
        self.movement_samples: list[dict] = []

        with open(data_path) as f:
            for line in f:
                sample = json.loads(line)
                if sample["is_drafting"]:
                    self.drafting_samples.append(sample)
                else:
                    self.movement_samples.append(sample)

        print(f"Loaded {len(self.drafting_samples)} drafting samples")
        print(f"Loaded {len(self.movement_samples)} movement samples")

    def __len__(self):
        return len(self.drafting_samples) + len(self.movement_samples)

    def get_drafting_data(self) -> tuple[torch.Tensor, torch.Tensor, torch.Tensor]:
        """Return all drafting data as tensors."""
        if not self.drafting_samples:
            return (
                torch.zeros(0, NUM_FEATURES),
                torch.zeros(0, NUM_CELLS),
                torch.zeros(0, 1),
            )

        features = torch.tensor(
            [s["features"] for s in self.drafting_samples], dtype=torch.float32
        )
        policies = torch.tensor(
            [s["policy"] for s in self.drafting_samples], dtype=torch.float32
        )
        values = torch.tensor(
            [[s["value"]] for s in self.drafting_samples], dtype=torch.float32
        )
        return features, policies, values

    def get_movement_data(self) -> tuple[torch.Tensor, torch.Tensor, torch.Tensor]:
        """Return all movement data as tensors."""
        if not self.movement_samples:
            return (
                torch.zeros(0, NUM_FEATURES),
                torch.zeros(0, MOVEMENT_POLICY_SIZE),
                torch.zeros(0, 1),
            )

        features = torch.tensor(
            [s["features"] for s in self.movement_samples], dtype=torch.float32
        )
        policies = torch.tensor(
            [s["policy"] for s in self.movement_samples], dtype=torch.float32
        )
        values = torch.tensor(
            [[s["value"]] for s in self.movement_samples], dtype=torch.float32
        )
        return features, policies, values


def features_to_grid(features: torch.Tensor) -> torch.Tensor:
    """Convert flat 480 features to 8x8x8 grid format for Conv2D.

    Input: (batch, 480) - 8 channels × 60 cells flattened
    Output: (batch, 8, 8, 8) - 8 channels × 8 rows × 8 cols
    """
    batch_size = features.shape[0]
    # Reshape to (batch, 8 channels, 60 cells)
    features = features.view(batch_size, NUM_CHANNELS, NUM_CELLS)

    # Create output grid (batch, channels, rows, cols)
    grid = torch.zeros(batch_size, NUM_CHANNELS, NUM_ROWS, NUM_COLS,
                       device=features.device, dtype=features.dtype)

    # Map each cell to its grid position
    for cell_idx, (row, col) in enumerate(CELL_TO_GRID):
        grid[:, :, row, col] = features[:, :, cell_idx]

    return grid


def grid_to_cells(grid: torch.Tensor) -> torch.Tensor:
    """Extract the 60 valid cells from an 8x8 grid.

    Input: (batch, 1, 8, 8) or (batch, 8, 8)
    Output: (batch, 60)
    """
    if grid.dim() == 4:
        grid = grid.squeeze(1)  # Remove channel dim

    batch_size = grid.shape[0]
    cells = torch.zeros(batch_size, NUM_CELLS, device=grid.device, dtype=grid.dtype)

    for cell_idx, (row, col) in enumerate(CELL_TO_GRID):
        cells[:, cell_idx] = grid[:, row, col]

    return cells


class ConvResidualBlock(nn.Module):
    """Residual block with two conv layers (OpenSpiel AlphaZero style)."""

    def __init__(self, channels: int):
        super().__init__()
        self.conv1 = nn.Conv2d(channels, channels, kernel_size=3, padding=1)
        self.bn1 = nn.BatchNorm2d(channels)
        self.conv2 = nn.Conv2d(channels, channels, kernel_size=3, padding=1)
        self.bn2 = nn.BatchNorm2d(channels)

    def forward(self, x):
        residual = x
        out = F.relu(self.bn1(self.conv1(x)))
        out = self.bn2(self.conv2(out))
        out = F.relu(out + residual)
        return out


class HTMFNet(nn.Module):
    """
    Neural network for HTMF with policy and value heads.

    Architecture matches OpenSpiel's AlphaZero ResNet:
    - Input: 8 channels × 8×8 grid (60 valid cells embedded in 64)
    - Shared trunk: Initial conv + residual blocks
    - Policy head: 1×1 conv → BN → ReLU → flatten → FC
    - Value head: 1×1 conv → BN → ReLU → flatten → FC → ReLU → FC → tanh
    """

    def __init__(self, policy_size: int, num_filters: int = 64, num_blocks: int = 4):
        super().__init__()
        self.policy_size = policy_size

        # Initial convolution
        self.input_conv = nn.Conv2d(NUM_CHANNELS, num_filters, kernel_size=3, padding=1)
        self.input_bn = nn.BatchNorm2d(num_filters)

        # Residual blocks
        self.blocks = nn.ModuleList([ConvResidualBlock(num_filters) for _ in range(num_blocks)])

        # Policy head: 1×1 conv to reduce channels, then flatten and FC
        self.policy_conv = nn.Conv2d(num_filters, 2, kernel_size=1)
        self.policy_bn = nn.BatchNorm2d(2)
        self.policy_fc = nn.Linear(2 * GRID_SIZE, policy_size)

        # Value head: 1×1 conv, flatten, FC layers
        self.value_conv = nn.Conv2d(num_filters, 1, kernel_size=1)
        self.value_bn = nn.BatchNorm2d(1)
        self.value_fc1 = nn.Linear(GRID_SIZE, num_filters)
        self.value_fc2 = nn.Linear(num_filters, 1)

    def forward(self, x):
        # Convert flat features to grid: (batch, 480) -> (batch, 8, 8, 8)
        x = features_to_grid(x)

        # Shared trunk
        x = F.relu(self.input_bn(self.input_conv(x)))
        for block in self.blocks:
            x = block(x)

        # Policy head
        policy = F.relu(self.policy_bn(self.policy_conv(x)))
        policy = policy.view(policy.size(0), -1)  # Flatten
        policy = self.policy_fc(policy)

        # Value head
        value = F.relu(self.value_bn(self.value_conv(x)))
        value = value.view(value.size(0), -1)  # Flatten
        value = F.relu(self.value_fc1(value))
        value = torch.tanh(self.value_fc2(value))

        return policy, value


def train_model(
    model: HTMFNet,
    features: torch.Tensor,
    policies: torch.Tensor,
    values: torch.Tensor,
    optimizer: torch.optim.Optimizer,
    device: torch.device,
    batch_size: int = 256,
) -> tuple[float, float]:
    """Train the model for one epoch and return (policy_loss, value_loss)."""
    model.train()

    if len(features) == 0:
        return 0.0, 0.0

    # Shuffle data
    perm = torch.randperm(len(features))
    features = features[perm].to(device)
    policies = policies[perm].to(device)
    # Convert values from [0, 1] to [-1, 1] for tanh output
    values = (values[perm] * 2 - 1).to(device)

    total_policy_loss = 0.0
    total_value_loss = 0.0
    num_batches = 0

    for i in range(0, len(features), batch_size):
        batch_features = features[i : i + batch_size]
        batch_policies = policies[i : i + batch_size]
        batch_values = values[i : i + batch_size]

        optimizer.zero_grad()

        pred_policy, pred_value = model(batch_features)

        # Policy loss: cross-entropy with target distribution
        # Target policies are probability distributions from MCTS
        policy_loss = -(batch_policies * F.log_softmax(pred_policy, dim=1)).sum(dim=1).mean()

        # Value loss: MSE
        value_loss = F.mse_loss(pred_value, batch_values)

        # Combined loss
        loss = policy_loss + value_loss
        loss.backward()
        optimizer.step()

        total_policy_loss += policy_loss.item()
        total_value_loss += value_loss.item()
        num_batches += 1

    return total_policy_loss / max(1, num_batches), total_value_loss / max(1, num_batches)


def export_to_onnx(model: HTMFNet, path: Path, device: torch.device):
    """Export model to ONNX format."""
    model.eval()
    # Move to CPU for ONNX export to avoid device compatibility issues
    model_cpu = model.cpu()
    dummy_input = torch.zeros(1, NUM_FEATURES)

    torch.onnx.export(
        model_cpu,
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
        dynamo=False,  # Use legacy exporter for compatibility
    )

    # Move model back to original device
    model.to(device)


def main():
    parser = argparse.ArgumentParser(description="Train HTMF neural network")
    parser.add_argument("--epochs", type=int, default=20, help="Number of training epochs")
    parser.add_argument("--lr", type=float, default=0.001, help="Learning rate")
    parser.add_argument("--batch-size", type=int, default=256, help="Batch size")
    parser.add_argument("--num-filters", type=int, default=64, help="Number of conv filters")
    parser.add_argument("--num-blocks", type=int, default=4, help="Number of residual blocks")
    args = parser.parse_args()

    ARTIFACTS_DIR.mkdir(parents=True, exist_ok=True)

    # Check for training data
    if not TRAINING_DATA.exists():
        print(f"Error: Training data not found at {TRAINING_DATA}")
        print("Run self-play first to generate training data.")
        return 1

    # Device selection
    if torch.cuda.is_available():
        device = torch.device("cuda")
        print("Using CUDA")
    elif torch.backends.mps.is_available():
        device = torch.device("mps")
        print("Using MPS (Apple Silicon)")
    else:
        device = torch.device("cpu")
        print("Using CPU")

    # Load data
    print(f"Loading training data from {TRAINING_DATA}...")
    dataset = HTMFDataset(TRAINING_DATA)

    drafting_features, drafting_policies, drafting_values = dataset.get_drafting_data()
    movement_features, movement_policies, movement_values = dataset.get_movement_data()

    print(f"Total samples: {len(dataset)}")

    # Create models
    drafting_model = HTMFNet(
        policy_size=NUM_CELLS, num_filters=args.num_filters, num_blocks=args.num_blocks
    ).to(device)
    movement_model = HTMFNet(
        policy_size=MOVEMENT_POLICY_SIZE, num_filters=args.num_filters, num_blocks=args.num_blocks
    ).to(device)

    # Load existing weights if available
    if MODEL_CHECKPOINT.exists():
        print(f"Loading existing model from {MODEL_CHECKPOINT}...")
        checkpoint = torch.load(MODEL_CHECKPOINT, map_location=device, weights_only=True)
        drafting_model.load_state_dict(checkpoint["drafting_model"])
        movement_model.load_state_dict(checkpoint["movement_model"])
        print("Loaded existing model weights")

    # Optimizers
    drafting_optimizer = torch.optim.Adam(drafting_model.parameters(), lr=args.lr)
    movement_optimizer = torch.optim.Adam(movement_model.parameters(), lr=args.lr)

    print(f"\nTraining for {args.epochs} epochs...")
    print(f"Learning rate: {args.lr}")
    print(f"Batch size: {args.batch_size}")
    print()

    for epoch in range(1, args.epochs + 1):
        # Train drafting model
        d_policy_loss, d_value_loss = train_model(
            drafting_model,
            drafting_features,
            drafting_policies,
            drafting_values,
            drafting_optimizer,
            device,
            args.batch_size,
        )

        # Train movement model
        m_policy_loss, m_value_loss = train_model(
            movement_model,
            movement_features,
            movement_policies,
            movement_values,
            movement_optimizer,
            device,
            args.batch_size,
        )

        print(
            f"Epoch {epoch:3d}/{args.epochs}: "
            f"Drafting [P: {d_policy_loss:.4f}, V: {d_value_loss:.4f}] | "
            f"Movement [P: {m_policy_loss:.4f}, V: {m_value_loss:.4f}]"
        )

    # Save PyTorch checkpoint
    print(f"\nSaving model to {MODEL_CHECKPOINT}...")
    torch.save(
        {
            "drafting_model": drafting_model.state_dict(),
            "movement_model": movement_model.state_dict(),
            "num_filters": args.num_filters,
            "num_blocks": args.num_blocks,
        },
        MODEL_CHECKPOINT,
    )

    # Export to ONNX
    print(f"Exporting drafting model to {ONNX_DRAFTING}...")
    export_to_onnx(drafting_model, ONNX_DRAFTING, device)

    print(f"Exporting movement model to {ONNX_MOVEMENT}...")
    export_to_onnx(movement_model, ONNX_MOVEMENT, device)

    print("\nTraining complete!")
    return 0


if __name__ == "__main__":
    exit(main())

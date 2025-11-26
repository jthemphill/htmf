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
NUM_FEATURES = 8 * NUM_CELLS  # 480
NUM_PENGUINS = 4
NUM_DIRECTIONS = 6
MAX_DISTANCE = 7
MOVEMENT_POLICY_SIZE = NUM_PENGUINS * NUM_DIRECTIONS * MAX_DISTANCE  # 168

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


class ResidualBlock(nn.Module):
    """Residual block with two fully connected layers."""

    def __init__(self, hidden_size: int):
        super().__init__()
        self.fc1 = nn.Linear(hidden_size, hidden_size)
        self.bn1 = nn.BatchNorm1d(hidden_size)
        self.fc2 = nn.Linear(hidden_size, hidden_size)
        self.bn2 = nn.BatchNorm1d(hidden_size)

    def forward(self, x):
        residual = x
        out = F.relu(self.bn1(self.fc1(x)))
        out = self.bn2(self.fc2(out))
        out = F.relu(out + residual)
        return out


class HTMFNet(nn.Module):
    """
    Neural network for HTMF with policy and value heads.

    Architecture:
    - Input: 480 features (8 channels × 60 cells)
    - Shared trunk: FC layers with residual connections
    - Policy head: outputs logits for moves
    - Value head: outputs win probability
    """

    def __init__(self, policy_size: int, hidden_size: int = 256, num_blocks: int = 4):
        super().__init__()

        # Input projection
        self.input_fc = nn.Linear(NUM_FEATURES, hidden_size)
        self.input_bn = nn.BatchNorm1d(hidden_size)

        # Residual blocks
        self.blocks = nn.ModuleList([ResidualBlock(hidden_size) for _ in range(num_blocks)])

        # Policy head
        self.policy_fc1 = nn.Linear(hidden_size, hidden_size)
        self.policy_bn = nn.BatchNorm1d(hidden_size)
        self.policy_fc2 = nn.Linear(hidden_size, policy_size)

        # Value head
        self.value_fc1 = nn.Linear(hidden_size, hidden_size // 2)
        self.value_bn = nn.BatchNorm1d(hidden_size // 2)
        self.value_fc2 = nn.Linear(hidden_size // 2, 1)

    def forward(self, x):
        # Shared trunk
        x = F.relu(self.input_bn(self.input_fc(x)))
        for block in self.blocks:
            x = block(x)

        # Policy head
        policy = F.relu(self.policy_bn(self.policy_fc1(x)))
        policy = self.policy_fc2(policy)

        # Value head
        value = F.relu(self.value_bn(self.value_fc1(x)))
        value = torch.sigmoid(self.value_fc2(value))

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
    values = values[perm].to(device)

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
    parser.add_argument("--hidden-size", type=int, default=256, help="Hidden layer size")
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
        policy_size=NUM_CELLS, hidden_size=args.hidden_size, num_blocks=args.num_blocks
    ).to(device)
    movement_model = HTMFNet(
        policy_size=MOVEMENT_POLICY_SIZE, hidden_size=args.hidden_size, num_blocks=args.num_blocks
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
            "hidden_size": args.hidden_size,
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

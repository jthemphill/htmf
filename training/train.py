#!/usr/bin/env python3
"""
Train a neural network for HTMF using PyTorch.

The network has two heads:
- Policy head: probability distribution over moves
  - Drafting: 60 values (one per cell)
  - Movement: 2520 values (60 source cells x 6 directions x 7 distances)
- Value head: predicted win probability for current player

Usage:
    uv run train.py [--epochs N] [--lr RATE] [--batch-size B]
"""

import argparse
import json
from collections import Counter
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
NUM_DIRECTIONS = 6
MAX_DISTANCE = 7
POLICY_VERSION = 2
MOVEMENT_POLICY_SIZE = NUM_CELLS * NUM_DIRECTIONS * MAX_DISTANCE  # 2520
VALID_TEACHERS = {"uniform", "nn_root"}

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
ONNX_MODEL = ARTIFACTS_DIR / "model.onnx"


class HTMFDataset(Dataset):
    """Dataset for HTMF training samples."""

    def __init__(self, data_path: Path):
        self.drafting_samples: list[dict] = []
        self.movement_samples: list[dict] = []
        self.teacher_counts: Counter[str] = Counter()

        with open(data_path) as f:
            for line_num, line in enumerate(f, start=1):
                sample = json.loads(line)
                policy_version = sample.get("policy_version")
                if policy_version != POLICY_VERSION:
                    raise ValueError(
                        f"{data_path}:{line_num}: expected policy_version={POLICY_VERSION}, "
                        f"got {policy_version!r}. Regenerate selfplay data; v1 movement "
                        "policies are not compatible with the absolute source-cell encoding."
                    )

                if "turn" not in sample:
                    raise ValueError(
                        f"{data_path}:{line_num}: missing turn metadata. Regenerate "
                        "policy-v2 selfplay data before training."
                    )

                teacher = sample.get("teacher")
                if teacher not in VALID_TEACHERS:
                    raise ValueError(
                        f"{data_path}:{line_num}: expected teacher in "
                        f"{sorted(VALID_TEACHERS)}, got {teacher!r}. Rebuild replay "
                        "so legacy samples are normalized."
                    )

                if not isinstance(sample.get("teacher_playouts"), int):
                    raise ValueError(
                        f"{data_path}:{line_num}: teacher_playouts must be an integer"
                    )

                if not isinstance(sample.get("model_prior_weight"), int | float):
                    raise ValueError(
                        f"{data_path}:{line_num}: model_prior_weight must be numeric"
                    )

                if not isinstance(sample.get("run_id"), str) or not sample["run_id"]:
                    raise ValueError(
                        f"{data_path}:{line_num}: run_id must be a non-empty string"
                    )

                if "teacher_model" in sample and not isinstance(sample["teacher_model"], str):
                    raise ValueError(
                        f"{data_path}:{line_num}: teacher_model must be a string when present"
                    )

                expected_policy_len = (
                    NUM_CELLS if sample["is_drafting"] else MOVEMENT_POLICY_SIZE
                )
                if len(sample["policy"]) != expected_policy_len:
                    raise ValueError(
                        f"{data_path}:{line_num}: expected policy length "
                        f"{expected_policy_len}, got {len(sample['policy'])}"
                    )

                self.teacher_counts[teacher] += 1
                if sample["is_drafting"]:
                    self.drafting_samples.append(sample)
                else:
                    self.movement_samples.append(sample)

        print(f"Loaded {len(self.drafting_samples)} drafting samples")
        print(f"Loaded {len(self.movement_samples)} movement samples")
        print(
            "Teacher counts: "
            + ", ".join(
                f"{teacher}={self.teacher_counts.get(teacher, 0)}"
                for teacher in sorted(VALID_TEACHERS)
            )
        )

    def __len__(self):
        return len(self.drafting_samples) + len(self.movement_samples)

    def get_drafting_data(
        self,
    ) -> tuple[
        torch.Tensor, torch.Tensor, torch.Tensor, torch.Tensor | None, torch.Tensor | None
    ]:
        """Return all drafting data as tensors.

        Returns:
            (features, policies, values, ownerships, score_diffs)
            Ownership and score_diff may be None if not present in training data.
        """
        if not self.drafting_samples:
            return (
                torch.zeros(0, NUM_FEATURES),
                torch.zeros(0, NUM_CELLS),
                torch.zeros(0, 1),
                None,
                None,
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

        # Check if auxiliary targets are available
        has_ownership = "ownership" in self.drafting_samples[0]
        has_score_diff = "score_diff" in self.drafting_samples[0]

        ownerships = None
        if has_ownership:
            ownerships = torch.tensor(
                [s["ownership"] for s in self.drafting_samples], dtype=torch.long
            )

        score_diffs = None
        if has_score_diff:
            score_diffs = torch.tensor(
                [s["score_diff"] for s in self.drafting_samples], dtype=torch.long
            )

        return features, policies, values, ownerships, score_diffs

    def get_movement_data(
        self,
    ) -> tuple[
        torch.Tensor, torch.Tensor, torch.Tensor, torch.Tensor | None, torch.Tensor | None
    ]:
        """Return all movement data as tensors.

        Returns:
            (features, policies, values, ownerships, score_diffs)
            Ownership and score_diff may be None if not present in training data.
        """
        if not self.movement_samples:
            return (
                torch.zeros(0, NUM_FEATURES),
                torch.zeros(0, MOVEMENT_POLICY_SIZE),
                torch.zeros(0, 1),
                None,
                None,
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

        # Check if auxiliary targets are available
        has_ownership = "ownership" in self.movement_samples[0]
        has_score_diff = "score_diff" in self.movement_samples[0]

        ownerships = None
        if has_ownership:
            ownerships = torch.tensor(
                [s["ownership"] for s in self.movement_samples], dtype=torch.long
            )

        score_diffs = None
        if has_score_diff:
            score_diffs = torch.tensor(
                [s["score_diff"] for s in self.movement_samples], dtype=torch.long
            )

        return features, policies, values, ownerships, score_diffs


def features_to_grid(features: torch.Tensor) -> torch.Tensor:
    """Convert flat 480 features to 8x8x8 grid format for Conv2D.

    Input: (batch, 480) - 8 channels × 60 cells flattened
    Output: (batch, 8, 8, 8) - 8 channels × 8 rows × 8 cols
    """
    batch_size = features.shape[0]
    # Reshape to (batch, 8 channels, 60 cells)
    features = features.view(batch_size, NUM_CHANNELS, NUM_CELLS)

    # Create output grid (batch, channels, rows, cols)
    grid = torch.zeros(
        batch_size,
        NUM_CHANNELS,
        NUM_ROWS,
        NUM_COLS,
        device=features.device,
        dtype=features.dtype,
    )

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


class SharedTrunk(nn.Module):
    """Shared convolutional trunk for processing board state."""

    def __init__(self, num_filters: int = 64, num_blocks: int = 4):
        super().__init__()

        # Initial convolution
        self.input_conv = nn.Conv2d(NUM_CHANNELS, num_filters, kernel_size=3, padding=1)
        self.input_bn = nn.BatchNorm2d(num_filters)

        # Residual blocks
        self.blocks = nn.ModuleList(
            [ConvResidualBlock(num_filters) for _ in range(num_blocks)]
        )

    def forward(self, x):
        # Convert flat features to grid: (batch, 480) -> (batch, 8, 8, 8)
        x = features_to_grid(x)

        # Shared trunk
        x = F.relu(self.input_bn(self.input_conv(x)))
        for block in self.blocks:
            x = block(x)

        return x


class PolicyHead(nn.Module):
    """Policy head for outputting move probabilities."""

    def __init__(self, num_filters: int, policy_size: int):
        super().__init__()
        self.policy_conv = nn.Conv2d(num_filters, 2, kernel_size=1)
        self.policy_bn = nn.BatchNorm2d(2)
        self.policy_fc = nn.Linear(2 * GRID_SIZE, policy_size)

    def forward(self, x):
        policy = F.relu(self.policy_bn(self.policy_conv(x)))
        policy = policy.view(policy.size(0), -1)  # Flatten
        policy = self.policy_fc(policy)
        return policy


class ValueHead(nn.Module):
    """Value head for predicting win probability."""

    def __init__(self, num_filters: int):
        super().__init__()
        self.value_conv = nn.Conv2d(num_filters, 1, kernel_size=1)
        self.value_bn = nn.BatchNorm2d(1)
        self.value_fc1 = nn.Linear(GRID_SIZE, num_filters)
        self.value_fc2 = nn.Linear(num_filters, 1)

    def forward(self, x):
        value = F.relu(self.value_bn(self.value_conv(x)))
        value = value.view(value.size(0), -1)  # Flatten
        value = F.relu(self.value_fc1(value))
        value = torch.tanh(self.value_fc2(value))
        return value


class OwnershipHead(nn.Module):
    """Ownership head for predicting which player owns each cell at game end.

    Following KataGo, this predicts per-cell ownership to provide localized
    gradient feedback for credit assignment.

    Output: (batch, 60, 3) where the 3 classes are [player_0, player_1, neither]
    """

    def __init__(self, num_filters: int):
        super().__init__()
        # Use convolutions to preserve spatial structure
        self.ownership_conv = nn.Conv2d(num_filters, 3, kernel_size=1)
        self.ownership_bn = nn.BatchNorm2d(3)

    def forward(self, x):
        # x is (batch, num_filters, 8, 8)
        ownership = self.ownership_bn(self.ownership_conv(x))  # (batch, 3, 8, 8)

        # Extract the 60 valid cells for each of the 3 classes
        batch_size = ownership.shape[0]
        ownership_cells = torch.zeros(
            batch_size, 3, NUM_CELLS, device=ownership.device, dtype=ownership.dtype
        )

        for cell_idx, (row, col) in enumerate(CELL_TO_GRID):
            ownership_cells[:, :, cell_idx] = ownership[:, :, row, col]

        # Reshape to (batch, 60, 3) for cross-entropy loss
        ownership_cells = ownership_cells.transpose(1, 2)  # (batch, 60, 3)

        return ownership_cells


class ScoreDifferenceHead(nn.Module):
    """Score difference head for predicting final score difference.

    Following KataGo, this predicts a distribution over possible score differences
    to provide finer-grained learning signal than binary win/loss.

    Score range: [-92, +92] → 185 possible values (inclusive)
    Output: (batch, 185) logits over score distribution
    """

    # Score difference range constants
    MIN_SCORE_DIFF = -92
    MAX_SCORE_DIFF = 92
    NUM_SCORE_BINS = MAX_SCORE_DIFF - MIN_SCORE_DIFF + 1  # 185

    def __init__(self, num_filters: int):
        super().__init__()
        self.score_conv = nn.Conv2d(num_filters, 1, kernel_size=1)
        self.score_bn = nn.BatchNorm2d(1)
        self.score_fc1 = nn.Linear(GRID_SIZE, num_filters)
        self.score_fc2 = nn.Linear(num_filters, self.NUM_SCORE_BINS)

    def forward(self, x):
        score = F.relu(self.score_bn(self.score_conv(x)))
        score = score.view(score.size(0), -1)  # Flatten
        score = F.relu(self.score_fc1(score))
        score = self.score_fc2(score)  # (batch, 185) logits
        return score


class HTMFNet(nn.Module):
    """
    Neural network for HTMF with shared trunk and multiple heads.

    Architecture:
    - Input: 8 channels x 8x8 grid (60 valid cells embedded in 64)
    - Shared trunk: Initial conv + residual blocks (shared between all heads)
    - Drafting policy head: outputs 60 cell probabilities
    - Movement policy head: outputs 2520 move probabilities (60 source cells × 6 dirs × 7 dists)
    - Value head: predicts win probability
    - Ownership head: predicts per-cell ownership at game end (60 cells × 3 classes)
    - Score difference head: predicts final score difference distribution (185 bins)
    """

    def __init__(self, num_filters: int = 64, num_blocks: int = 4):
        super().__init__()

        # Shared convolutional trunk
        self.trunk = SharedTrunk(num_filters, num_blocks)

        # Policy heads for drafting and movement
        self.drafting_policy = PolicyHead(num_filters, NUM_CELLS)
        self.movement_policy = PolicyHead(num_filters, MOVEMENT_POLICY_SIZE)

        # Main value head
        self.value = ValueHead(num_filters)

        # Auxiliary heads (KataGo-style)
        self.ownership = OwnershipHead(num_filters)
        self.score_diff = ScoreDifferenceHead(num_filters)

    def forward(self, x, is_drafting: bool | None = None):
        """Forward pass.

        Args:
            x: Input features (batch, 480)
            is_drafting: If True, return only drafting policy; if False, return only movement policy;
                        if None, return both policies (for ONNX export)

        Returns:
            - If is_drafting is True/False: (policy, value, ownership, score_diff) tuple
            - If is_drafting is None: (drafting_policy, movement_policy, value, ownership, score_diff) tuple
        """
        # Shared trunk
        trunk_out = self.trunk(x)

        # Auxiliary heads (always computed)
        value = self.value(trunk_out)
        ownership = self.ownership(trunk_out)
        score_diff = self.score_diff(trunk_out)

        # Select appropriate policy head(s)
        if is_drafting is None:
            # Return both policies (for ONNX export)
            drafting_policy = self.drafting_policy(trunk_out)
            movement_policy = self.movement_policy(trunk_out)
            return drafting_policy, movement_policy, value, ownership, score_diff
        elif is_drafting:
            policy = self.drafting_policy(trunk_out)
        else:
            policy = self.movement_policy(trunk_out)

        return policy, value, ownership, score_diff


def score_diff_to_index(score_diff: int) -> int:
    """Convert score difference to bin index.

    Args:
        score_diff: Score difference in range [-92, 92]

    Returns:
        Bin index in range [0, 184]
    """
    return score_diff - ScoreDifferenceHead.MIN_SCORE_DIFF


def train_model(
    model: HTMFNet,
    features: torch.Tensor,
    policies: torch.Tensor,
    values: torch.Tensor,
    ownerships: torch.Tensor | None,
    score_diffs: torch.Tensor | None,
    optimizer: torch.optim.Optimizer,
    device: torch.device,
    is_drafting: bool,
    batch_size: int = 256,
    value_weight: float = 0.25,
    aux_weight: float = 0.05,
) -> tuple[float, float, float, float]:
    """Train the model for one epoch.

    Returns:
        (policy_loss, value_loss, ownership_loss, score_diff_loss)
    """
    model.train()

    if len(features) == 0:
        return 0.0, 0.0, 0.0, 0.0

    # Shuffle data
    perm = torch.randperm(len(features))
    features = features[perm].to(device)
    policies = policies[perm].to(device)
    # Convert values from [0, 1] to [-1, 1] for tanh output
    values = (values[perm] * 2 - 1).to(device)

    # Auxiliary targets (may be None if not available)
    if ownerships is not None:
        ownerships = ownerships[perm].to(device)
    if score_diffs is not None:
        score_diffs = score_diffs[perm].to(device)

    total_policy_loss = 0.0
    total_value_loss = 0.0
    total_ownership_loss = 0.0
    total_score_diff_loss = 0.0
    num_batches = 0

    for i in range(0, len(features), batch_size):
        batch_features = features[i : i + batch_size]
        batch_policies = policies[i : i + batch_size]
        batch_values = values[i : i + batch_size]

        optimizer.zero_grad()

        pred_policy, pred_value, pred_ownership, pred_score_diff = model(
            batch_features, is_drafting
        )

        # Policy loss: cross-entropy with target distribution
        # Target policies are probability distributions from MCTS
        policy_loss = (
            -(batch_policies * F.log_softmax(pred_policy, dim=1)).sum(dim=1).mean()
        )

        # Value loss: MSE
        value_loss = F.mse_loss(pred_value, batch_values)

        # Initialize auxiliary losses
        ownership_loss = torch.tensor(0.0, device=device)
        score_diff_loss = torch.tensor(0.0, device=device)

        # Ownership loss: per-cell cross-entropy
        if ownerships is not None:
            batch_ownerships = ownerships[i : i + batch_size]
            # pred_ownership: (batch, 60, 3)
            # batch_ownerships: (batch, 60) with class indices [0, 1, 2]
            ownership_loss = F.cross_entropy(
                pred_ownership.reshape(-1, 3),  # (batch * 60, 3)
                batch_ownerships.reshape(-1).long(),  # (batch * 60,)
            )

        # Score difference loss: PDF + CDF loss (KataGo-style)
        if score_diffs is not None:
            batch_score_diffs = score_diffs[i : i + batch_size]
            # pred_score_diff: (batch, 185) logits
            # batch_score_diffs: (batch,) with bin indices [0, 184]

            # PDF loss: standard cross-entropy with one-hot target
            pdf_loss = F.cross_entropy(pred_score_diff, batch_score_diffs.long())

            # CDF loss: penalize cumulative distribution error
            # Compute predicted and target CDFs
            pred_probs = F.softmax(pred_score_diff, dim=1)
            pred_cdf = torch.cumsum(pred_probs, dim=1)

            # Create target CDF (step function at true score)
            target_cdf = torch.zeros_like(pred_cdf)
            for j, score_idx in enumerate(batch_score_diffs):
                target_cdf[j, int(score_idx) :] = 1.0

            # MSE between CDFs
            cdf_loss = F.mse_loss(pred_cdf, target_cdf)

            # Combine PDF and CDF losses (equal weighting as in KataGo)
            score_diff_loss = pdf_loss + cdf_loss

        # Policy priors are the only v1 search signal. Value and auxiliary heads
        # stay as regularizers, weighted low enough not to drown out policy loss.
        loss = (
            policy_loss
            + value_weight * value_loss
            + aux_weight * ownership_loss
            + aux_weight * score_diff_loss
        )
        loss.backward()
        optimizer.step()

        total_policy_loss += policy_loss.item()
        total_value_loss += value_loss.item()
        total_ownership_loss += ownership_loss.item()
        total_score_diff_loss += score_diff_loss.item()
        num_batches += 1

    avg_policy = total_policy_loss / max(1, num_batches)
    avg_value = total_value_loss / max(1, num_batches)
    avg_ownership = total_ownership_loss / max(1, num_batches)
    avg_score_diff = total_score_diff_loss / max(1, num_batches)

    return avg_policy, avg_value, avg_ownership, avg_score_diff


def export_to_onnx(model: HTMFNet, path: Path):
    """Export model to ONNX format with all heads.

    The exported model has:
    - Input: features (batch, 480)
    - Outputs:
      - drafting_policy (batch, 60)
      - movement_policy (batch, 2520)
      - value (batch, 1)
      - ownership (batch, 60, 3) - per-cell ownership prediction
      - score_diff (batch, 185) - score difference distribution
    """
    # Move model to CPU for ONNX export (required for compatibility)
    model = model.cpu()
    model.eval()
    dummy_input = torch.zeros(1, NUM_FEATURES)

    # Create a wrapper that always outputs all heads
    class ONNXWrapper(nn.Module):
        def __init__(self, model):
            super().__init__()
            self.model = model

        def forward(self, x):
            return self.model(x, is_drafting=None)

    wrapper = ONNXWrapper(model)

    torch.onnx.export(
        wrapper,
        dummy_input,
        path,
        input_names=["features"],
        output_names=[
            "drafting_policy",
            "movement_policy",
            "value",
            "ownership",
            "score_diff",
        ],
        dynamo=False,  # Use legacy exporter for Python 3.14 compatibility
    )


def compatible_state_dict(
    model: HTMFNet, state_dict: dict[str, torch.Tensor]
) -> tuple[dict[str, torch.Tensor], list[str]]:
    """Return only checkpoint tensors whose names and shapes match this model."""
    current_state = model.state_dict()
    compatible = {}
    skipped = []

    for name, tensor in state_dict.items():
        if name in current_state and current_state[name].shape == tensor.shape:
            compatible[name] = tensor
        else:
            skipped.append(name)

    return compatible, skipped


def main():
    parser = argparse.ArgumentParser(description="Train HTMF neural network")
    parser.add_argument(
        "--epochs", type=int, default=20, help="Number of training epochs"
    )
    parser.add_argument("--lr", type=float, default=0.001, help="Learning rate")
    parser.add_argument("--batch-size", type=int, default=256, help="Batch size")
    parser.add_argument(
        "--num-filters",
        type=int,
        default=None,
        help="Number of conv filters; defaults to checkpoint metadata or 64 for a fresh model",
    )
    parser.add_argument(
        "--num-blocks",
        type=int,
        default=None,
        help="Number of residual blocks; defaults to checkpoint metadata or 4 for a fresh model",
    )
    parser.add_argument(
        "--value-weight",
        type=float,
        default=0.25,
        help="Loss weight for the value head; policy remains the primary search signal",
    )
    parser.add_argument(
        "--aux-weight",
        type=float,
        default=0.05,
        help="Loss weight for ownership and score-difference auxiliary heads",
    )
    parser.add_argument(
        "--fresh",
        action="store_true",
        help="Train from random initialization instead of loading model_final.pt",
    )
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

    (
        drafting_features,
        drafting_policies,
        drafting_values,
        drafting_ownerships,
        drafting_score_diffs,
    ) = dataset.get_drafting_data()
    (
        movement_features,
        movement_policies,
        movement_values,
        movement_ownerships,
        movement_score_diffs,
    ) = dataset.get_movement_data()

    print(f"Total samples: {len(dataset)}")
    if drafting_ownerships is not None or movement_ownerships is not None:
        print("Auxiliary targets detected: ownership, score_diff")

    checkpoint = None
    state_dict = None
    if args.fresh:
        print("Fresh training requested; not loading existing checkpoint.")
    elif MODEL_CHECKPOINT.exists():
        print(f"Loading existing model metadata from {MODEL_CHECKPOINT}...")
        checkpoint = torch.load(
            MODEL_CHECKPOINT, map_location=device, weights_only=True
        )
        if isinstance(checkpoint, dict) and "model" in checkpoint:
            state_dict = checkpoint["model"]
            if args.num_filters is None and "num_filters" in checkpoint:
                args.num_filters = int(checkpoint["num_filters"])
            if args.num_blocks is None and "num_blocks" in checkpoint:
                args.num_blocks = int(checkpoint["num_blocks"])
        else:
            state_dict = checkpoint

    if args.num_filters is None:
        args.num_filters = 64
    if args.num_blocks is None:
        args.num_blocks = 4

    # Create single shared model
    model = HTMFNet(num_filters=args.num_filters, num_blocks=args.num_blocks).to(device)

    # Load existing weights if available
    if state_dict is not None:
        compatible, skipped = compatible_state_dict(model, state_dict)
        missing, unexpected = model.load_state_dict(compatible, strict=False)
        print(
            f"Loaded {len(compatible)} tensors from existing checkpoint "
            f"({len(skipped)} shape/name mismatches skipped)"
        )
        if missing:
            print(f"Initialized {len(missing)} tensors from scratch")
        if unexpected:
            print(f"Ignored {len(unexpected)} unexpected tensors")

    # Single optimizer for the entire model
    optimizer = torch.optim.Adam(model.parameters(), lr=args.lr)

    # Learning rate scheduler: cosine annealing with warmup
    # Warmup for first 10% of epochs, then cosine decay
    warmup_epochs = max(1, args.epochs // 10)
    scheduler = torch.optim.lr_scheduler.CosineAnnealingLR(
        optimizer, T_max=args.epochs - warmup_epochs, eta_min=args.lr * 0.1
    )

    print(f"\nTraining for {args.epochs} epochs...")
    print(f"Learning rate: {args.lr} (with cosine annealing schedule)")
    print(f"Warmup epochs: {warmup_epochs}")
    print(f"Batch size: {args.batch_size}")
    print(f"Value loss weight: {args.value_weight}")
    print(f"Auxiliary loss weight: {args.aux_weight}")
    print()

    for epoch in range(1, args.epochs + 1):
        # Train on drafting data
        d_policy_loss, d_value_loss, d_ownership_loss, d_score_diff_loss = train_model(
            model,
            drafting_features,
            drafting_policies,
            drafting_values,
            drafting_ownerships,
            drafting_score_diffs,
            optimizer,
            device,
            is_drafting=True,
            batch_size=args.batch_size,
            value_weight=args.value_weight,
            aux_weight=args.aux_weight,
        )

        # Train on movement data
        m_policy_loss, m_value_loss, m_ownership_loss, m_score_diff_loss = train_model(
            model,
            movement_features,
            movement_policies,
            movement_values,
            movement_ownerships,
            movement_score_diffs,
            optimizer,
            device,
            is_drafting=False,
            batch_size=args.batch_size,
            value_weight=args.value_weight,
            aux_weight=args.aux_weight,
        )

        # Print detailed loss analysis
        print(f"\n{'='*80}")
        print(f"Epoch {epoch:3d}/{args.epochs}")
        print(f"{'='*80}")

        # Drafting losses
        print(f"\nDRAFTING PHASE:")
        print(f"  Raw losses:      P={d_policy_loss:.4f}  V={d_value_loss:.4f}  O={d_ownership_loss:.4f}  S={d_score_diff_loss:.4f}")
        if d_ownership_loss > 0:
            print(f"  Weighted:        V={args.value_weight*d_value_loss:.4f}  O={args.aux_weight*d_ownership_loss:.4f}  S={args.aux_weight*d_score_diff_loss:.4f}")
            total_weighted = d_policy_loss + args.value_weight*d_value_loss + args.aux_weight*d_ownership_loss + args.aux_weight*d_score_diff_loss
            print(f"  Total weighted loss: {total_weighted:.4f}")
            print(f"  Contribution %:  P={100*d_policy_loss/total_weighted:.1f}%  V={100*args.value_weight*d_value_loss/total_weighted:.1f}%  O={100*args.aux_weight*d_ownership_loss/total_weighted:.1f}%  S={100*args.aux_weight*d_score_diff_loss/total_weighted:.1f}%")

        # Movement losses
        print(f"\nMOVEMENT PHASE:")
        print(f"  Raw losses:      P={m_policy_loss:.4f}  V={m_value_loss:.4f}  O={m_ownership_loss:.4f}  S={m_score_diff_loss:.4f}")
        if m_ownership_loss > 0:
            print(f"  Weighted:        V={args.value_weight*m_value_loss:.4f}  O={args.aux_weight*m_ownership_loss:.4f}  S={args.aux_weight*m_score_diff_loss:.4f}")
            total_weighted = m_policy_loss + args.value_weight*m_value_loss + args.aux_weight*m_ownership_loss + args.aux_weight*m_score_diff_loss
            print(f"  Total weighted loss: {total_weighted:.4f}")
            print(f"  Contribution %:  P={100*m_policy_loss/total_weighted:.1f}%  V={100*args.value_weight*m_value_loss/total_weighted:.1f}%  O={100*args.aux_weight*m_ownership_loss/total_weighted:.1f}%  S={100*args.aux_weight*m_score_diff_loss/total_weighted:.1f}%")

        # Update learning rate (warmup for first N epochs, then cosine annealing)
        current_lr = optimizer.param_groups[0]['lr']
        if epoch <= warmup_epochs:
            # Linear warmup
            new_lr = args.lr * (epoch / warmup_epochs)
            for param_group in optimizer.param_groups:
                param_group['lr'] = new_lr
            print(f"\nLearning rate: {new_lr:.6f} (warmup {epoch}/{warmup_epochs})")
        else:
            scheduler.step()
            current_lr = optimizer.param_groups[0]['lr']
            print(f"\nLearning rate: {current_lr:.6f} (cosine annealing)")

    # Save PyTorch checkpoint
    print(f"\nSaving model to {MODEL_CHECKPOINT}...")
    torch.save(
        {
            "model": model.state_dict(),
            "num_filters": args.num_filters,
            "num_blocks": args.num_blocks,
            "policy_version": POLICY_VERSION,
        },
        MODEL_CHECKPOINT,
    )

    # Export to ONNX (single file with both policy heads)
    print(f"Exporting model to {ONNX_MODEL}...")
    export_to_onnx(model, ONNX_MODEL)

    print("\nTraining complete!")
    return 0


if __name__ == "__main__":
    exit(main())

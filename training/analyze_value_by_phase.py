#!/usr/bin/env python3
"""
Analyze value head accuracy by game phase.

Key question: Is the value head actually learning strategy, or just counting points?

This script:
- Loads training data and model
- Buckets samples by game phase (early/mid/late based on claimed cells)
- Calculates value prediction accuracy for each phase
- Shows whether the value head has real predictive power early in the game
"""

import argparse
import json
from pathlib import Path

import numpy as np
import torch
import torch.nn.functional as F
from train import HTMFDataset, HTMFNet, MODEL_CHECKPOINT, TRAINING_DATA, NUM_CELLS


def get_game_phase(features, is_drafting):
    """
    Determine game phase based on how many cells are claimed.

    Returns:
        - "drafting": Still placing penguins
        - "early": 0-20% of cells claimed
        - "mid": 20-60% of cells claimed
        - "late": 60-100% of cells claimed
    """
    if is_drafting:
        return "drafting"

    # Count claimed cells (channels 5 and 6)
    claimed_p0 = features[5 * NUM_CELLS : 6 * NUM_CELLS].sum()
    claimed_p1 = features[6 * NUM_CELLS : 7 * NUM_CELLS].sum()
    total_claimed = claimed_p0 + claimed_p1

    pct_claimed = total_claimed / NUM_CELLS

    if pct_claimed < 0.2:
        return "early"
    elif pct_claimed < 0.6:
        return "mid"
    else:
        return "late"


def analyze_value_head(model, dataset, device):
    """Analyze value head predictions by game phase."""
    model.eval()

    # Collect all data
    all_samples = {
        "drafting": [],
        "early": [],
        "mid": [],
        "late": [],
    }

    # Process drafting samples
    drafting_features, _, drafting_values, _, _ = dataset.get_drafting_data()
    for i in range(len(drafting_features)):
        phase = get_game_phase(drafting_features[i], is_drafting=True)
        all_samples[phase].append({
            "features": drafting_features[i],
            "value": drafting_values[i],
            "is_drafting": True,
        })

    # Process movement samples
    movement_features, _, movement_values, _, _ = dataset.get_movement_data()
    for i in range(len(movement_features)):
        phase = get_game_phase(movement_features[i], is_drafting=False)
        all_samples[phase].append({
            "features": movement_features[i],
            "value": movement_values[i],
            "is_drafting": False,
        })

    print("="*80)
    print("VALUE HEAD ACCURACY BY GAME PHASE")
    print("="*80)
    print()

    # Analyze each phase
    for phase in ["drafting", "early", "mid", "late"]:
        samples = all_samples[phase]

        if len(samples) == 0:
            print(f"{phase.upper():>10}: No samples")
            continue

        # Prepare batch
        features = torch.stack([s["features"] for s in samples]).to(device)
        # Convert from [0, 1] to [-1, 1] for tanh output
        true_values = torch.stack([s["value"] for s in samples]).to(device)
        true_values_tanh = true_values * 2 - 1

        is_drafting = samples[0]["is_drafting"]

        with torch.no_grad():
            _, pred_value, _, _ = model(features, is_drafting)
            # pred_value is in [-1, 1], convert to [0, 1]
            pred_value_prob = (pred_value + 1.0) / 2.0

            # Calculate metrics
            mse = F.mse_loss(pred_value.squeeze(), true_values_tanh).item()
            mae = (pred_value_prob.squeeze() - true_values).abs().mean().item()

            # Accuracy: % within 0.1 of true value
            within_01 = ((pred_value_prob.squeeze() - true_values).abs() < 0.1).float().mean().item()
            within_02 = ((pred_value_prob.squeeze() - true_values).abs() < 0.2).float().mean().item()

            # Classification accuracy (win/loss/draw)
            # Bins: [0, 0.4) = loss, [0.4, 0.6] = draw, (0.6, 1] = win
            def classify(v):
                return torch.where(v < 0.4, 0, torch.where(v > 0.6, 2, 1))

            pred_class = classify(pred_value_prob.squeeze())
            true_class = classify(true_values)
            class_acc = (pred_class == true_class).float().mean().item()

            # Distribution of predictions and targets
            pred_mean = pred_value_prob.mean().item()
            true_mean = true_values.mean().item()
            pred_std = pred_value_prob.std().item()

        print(f"{phase.upper():>10} ({len(samples):>5} samples):")
        print(f"  MSE:                  {mse:.6f}")
        print(f"  MAE:                  {mae:.4f}")
        print(f"  Within ±0.1:          {within_01*100:.1f}%")
        print(f"  Within ±0.2:          {within_02*100:.1f}%")
        print(f"  Win/Draw/Loss acc:    {class_acc*100:.1f}%")
        print(f"  Prediction mean:      {pred_mean:.3f} (true: {true_mean:.3f})")
        print(f"  Prediction std:       {pred_std:.3f}")
        print()

    print("="*80)
    print("INTERPRETATION")
    print("="*80)
    print()
    print("Key questions:")
    print("1. Does accuracy drop significantly in early game vs late game?")
    print("   → If yes: value head is just counting points, not learning strategy")
    print("   → If no: value head has real predictive power")
    print()
    print("2. Is early game prediction mean ~0.5 (uncertain) or polarized?")
    print("   → Mean ~0.5: value head is guessing (bad)")
    print("   → Polarized distribution: value head sees strategic differences (good)")
    print()
    print("3. Is MAE in early game < 0.2?")
    print("   → MAE > 0.3: essentially random guessing")
    print("   → MAE < 0.2: has some predictive signal")
    print()


def main():
    parser = argparse.ArgumentParser(description="Analyze value head by game phase")
    args = parser.parse_args()

    # Check for required files
    if not TRAINING_DATA.exists():
        print(f"Error: Training data not found at {TRAINING_DATA}")
        print("Run self-play first to generate training data.")
        return 1

    if not MODEL_CHECKPOINT.exists():
        print(f"Error: Model checkpoint not found at {MODEL_CHECKPOINT}")
        return 1

    # Device selection
    if torch.cuda.is_available():
        device = torch.device("cuda")
    elif torch.backends.mps.is_available():
        device = torch.device("mps")
    else:
        device = torch.device("cpu")

    print(f"Using device: {device}")
    print()

    # Load data
    print(f"Loading training data from {TRAINING_DATA}...")
    dataset = HTMFDataset(TRAINING_DATA)

    # Load model
    print(f"Loading model from {MODEL_CHECKPOINT}...")
    checkpoint = torch.load(MODEL_CHECKPOINT, map_location=device, weights_only=True)

    # Extract model hyperparameters
    if "num_filters" in checkpoint and "num_blocks" in checkpoint:
        num_filters = checkpoint["num_filters"]
        num_blocks = checkpoint["num_blocks"]
        state_dict = checkpoint["model"]
    else:
        num_filters = 64
        num_blocks = 4
        state_dict = checkpoint

    print(f"Model architecture: {num_filters} filters, {num_blocks} blocks")
    print()

    model = HTMFNet(num_filters=num_filters, num_blocks=num_blocks).to(device)
    model.load_state_dict(state_dict)

    # Analyze
    analyze_value_head(model, dataset, device)

    return 0


if __name__ == "__main__":
    exit(main())

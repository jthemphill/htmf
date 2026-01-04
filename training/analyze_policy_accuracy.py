#!/usr/bin/env python3
"""
Analyze policy head accuracy on training data.

This calculates:
- Top-1 accuracy: % of times the highest probability move matches the MCTS target
- Top-3 accuracy: % of times the target move is in the top 3 predictions
- Top-5 accuracy: % of times the target move is in the top 5 predictions
- Cross-entropy loss (for comparison with training logs)
"""

import argparse
import json
from pathlib import Path

import numpy as np
import torch
import torch.nn.functional as F
from train import HTMFDataset, HTMFNet, MODEL_CHECKPOINT, TRAINING_DATA


def calculate_accuracy(model, features, policies, is_drafting, device):
    """Calculate top-k accuracy for policy predictions."""
    model.eval()

    with torch.no_grad():
        features = features.to(device)
        policies = policies.to(device)

        # Forward pass
        pred_policy, _, _, _ = model(features, is_drafting)
        pred_probs = F.softmax(pred_policy, dim=1)

        # Get target move (highest probability in MCTS policy)
        target_moves = torch.argmax(policies, dim=1)

        # Get top-k predictions
        top1_preds = torch.argmax(pred_probs, dim=1)
        _, top3_preds = torch.topk(pred_probs, k=min(3, pred_probs.shape[1]), dim=1)
        _, top5_preds = torch.topk(pred_probs, k=min(5, pred_probs.shape[1]), dim=1)

        # Calculate accuracies
        top1_correct = (top1_preds == target_moves).sum().item()
        top3_correct = sum(
            target_moves[i] in top3_preds[i] for i in range(len(target_moves))
        )
        top5_correct = sum(
            target_moves[i] in top5_preds[i] for i in range(len(target_moves))
        )

        total = len(target_moves)

        # Calculate cross-entropy loss (for verification)
        ce_loss = -(policies * F.log_softmax(pred_policy, dim=1)).sum(dim=1).mean().item()

        # Calculate KL divergence (another useful metric)
        kl_div = F.kl_div(
            F.log_softmax(pred_policy, dim=1),
            policies,
            reduction='batchmean'
        ).item()

        return {
            'top1_accuracy': 100.0 * top1_correct / total,
            'top3_accuracy': 100.0 * top3_correct / total,
            'top5_accuracy': 100.0 * top5_correct / total,
            'cross_entropy': ce_loss,
            'kl_divergence': kl_div,
            'num_samples': total,
        }


def main():
    parser = argparse.ArgumentParser(description="Analyze policy head accuracy")
    parser.add_argument(
        "--batch-size", type=int, default=1024, help="Batch size for evaluation"
    )
    args = parser.parse_args()

    # Check for required files
    if not TRAINING_DATA.exists():
        print(f"Error: Training data not found at {TRAINING_DATA}")
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
        # Old format: default hyperparameters
        num_filters = 64
        num_blocks = 4
        state_dict = checkpoint

    print(f"Model architecture: {num_filters} filters, {num_blocks} blocks")

    model = HTMFNet(num_filters=num_filters, num_blocks=num_blocks).to(device)
    model.load_state_dict(state_dict)

    print()
    print("="*80)
    print("POLICY HEAD ACCURACY ANALYSIS")
    print("="*80)

    # Analyze drafting phase
    print("\nDRAFTING PHASE (60 possible moves):")
    print("-" * 80)
    drafting_features, drafting_policies, _, _, _ = dataset.get_drafting_data()

    if len(drafting_features) > 0:
        drafting_metrics = calculate_accuracy(
            model, drafting_features, drafting_policies, is_drafting=True, device=device
        )

        print(f"  Samples:          {drafting_metrics['num_samples']}")
        print(f"  Top-1 accuracy:   {drafting_metrics['top1_accuracy']:.2f}%")
        print(f"  Top-3 accuracy:   {drafting_metrics['top3_accuracy']:.2f}%")
        print(f"  Top-5 accuracy:   {drafting_metrics['top5_accuracy']:.2f}%")
        print(f"  Cross-entropy:    {drafting_metrics['cross_entropy']:.4f}")
        print(f"  KL divergence:    {drafting_metrics['kl_divergence']:.4f}")
    else:
        print("  No drafting samples found")

    # Analyze movement phase
    print("\nMOVEMENT PHASE (168 possible moves max):")
    print("-" * 80)
    movement_features, movement_policies, _, _, _ = dataset.get_movement_data()

    if len(movement_features) > 0:
        movement_metrics = calculate_accuracy(
            model, movement_features, movement_policies, is_drafting=False, device=device
        )

        print(f"  Samples:          {movement_metrics['num_samples']}")
        print(f"  Top-1 accuracy:   {movement_metrics['top1_accuracy']:.2f}%")
        print(f"  Top-3 accuracy:   {movement_metrics['top3_accuracy']:.2f}%")
        print(f"  Top-5 accuracy:   {movement_metrics['top5_accuracy']:.2f}%")
        print(f"  Cross-entropy:    {movement_metrics['cross_entropy']:.4f}")
        print(f"  KL divergence:    {movement_metrics['kl_divergence']:.4f}")
    else:
        print("  No movement samples found")

    print()
    print("="*80)
    print("INTERPRETATION")
    print("="*80)
    print()
    print("Top-1 accuracy measures how often the model's #1 choice matches MCTS's #1 choice.")
    print("This is the most important metric for pure policy performance.")
    print()
    print("Top-3/5 accuracy shows whether the model at least considers the right moves,")
    print("even if it doesn't rank them perfectly.")
    print()
    print("Cross-entropy measures how well the full probability distribution matches MCTS.")
    print("Lower is better (should match training logs).")
    print()
    print("KL divergence measures distribution mismatch (0 = perfect match).")
    print()

    return 0


if __name__ == "__main__":
    exit(main())

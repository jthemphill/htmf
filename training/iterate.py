#!/usr/bin/env python3
"""
Iterative Training Loop for HTMF

This implements an AlphaZero-style training loop:
1. Generate self-play games using NN-guided MCTS
2. Train on the new data
3. Evaluate the new model against the old one
4. If better, replace the model and repeat

Usage:
    uv run iterate.py [--iterations N] [--games G] [--playouts P] [--epochs E]
"""

import argparse
import os
import shutil
import subprocess
import sys
from datetime import datetime
from pathlib import Path

# Paths
ARTIFACTS_DIR = Path("./artifacts")
TRAINING_DATA = ARTIFACTS_DIR / "training_data.jsonl"
MODEL_FINAL = ARTIFACTS_DIR / "model_final.pt"
ONNX_DRAFTING = ARTIFACTS_DIR / "model_drafting.onnx"
ONNX_MOVEMENT = ARTIFACTS_DIR / "model_movement.onnx"
ITERATIONS_DIR = ARTIFACTS_DIR / "iterations"


def run_command(cmd: list[str], cwd: str | None = None) -> subprocess.CompletedProcess:
    """Run a command and stream output."""
    print(f"$ {' '.join(cmd)}")
    result = subprocess.run(cmd, cwd=cwd)
    if result.returncode != 0:
        print(f"Command failed with exit code {result.returncode}")
    return result


def generate_selfplay_data(num_games: int, num_playouts: int, use_nn: bool) -> Path:
    """Generate self-play training data."""
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    output_file = ARTIFACTS_DIR / f"selfplay_{timestamp}.jsonl"

    cmd = [
        "cargo",
        "run",
        "--release",
        "-p",
        "selfplay",
        "--",
        str(num_games),
        str(num_playouts),
    ]
    if use_nn:
        cmd.append("--nn")

    print(f"\nGenerating {num_games} self-play games...")
    with open(output_file, "w") as f:
        result = subprocess.run(cmd, stdout=f, cwd=str(Path(__file__).parent.parent))

    if result.returncode != 0:
        print(f"Self-play failed with exit code {result.returncode}")
        sys.exit(1)

    # Count samples
    with open(output_file) as f:
        num_samples = sum(1 for _ in f)
    print(f"Generated {num_samples} training samples -> {output_file}")

    return output_file


def merge_training_data(new_data: Path, max_samples: int = 100_000):
    """Merge new data into the main training file, keeping recent samples."""
    # Read existing data
    existing_samples = []
    if TRAINING_DATA.exists():
        with open(TRAINING_DATA) as f:
            existing_samples = f.readlines()

    # Read new data
    with open(new_data) as f:
        new_samples = f.readlines()

    # Combine, keeping most recent samples
    all_samples = existing_samples + new_samples
    if len(all_samples) > max_samples:
        # Keep the most recent samples (at the end)
        all_samples = all_samples[-max_samples:]

    # Write back
    with open(TRAINING_DATA, "w") as f:
        f.writelines(all_samples)

    print(f"Training data: {len(all_samples)} samples (added {len(new_samples)} new)")


def train_model(epochs: int, learning_rate: float = 0.001) -> bool:
    """Train the model and return True if it improved."""
    print(f"\nTraining for {epochs} epochs...")
    result = run_command(
        [
            "uv",
            "run",
            "train.py",
            "--epochs",
            str(epochs),
            "--lr",
            str(learning_rate),
        ],
        cwd=str(Path(__file__).parent),
    )

    return result.returncode == 0


def evaluate_models(
    num_games: int = 20, num_playouts: int = 100
) -> tuple[int, int, int]:
    """
    Evaluate new model vs old model.
    Returns (new_wins, old_wins, draws)
    """
    print(f"\nEvaluating new model ({num_games} games, {num_playouts} playouts)...")
    result = subprocess.run(
        ["cargo", "run", "--release", "--bin", "nn_vs_mcts", "--"],
        capture_output=True,
        text=True,
        cwd=str(Path(__file__).parent.parent),
    )

    # Parse results from output
    # Looking for lines like "NN wins: X (Y%)"
    nn_wins = 0
    mcts_wins = 0
    draws = 0

    for line in result.stdout.split("\n") + result.stderr.split("\n"):
        if "NN wins:" in line:
            nn_wins = int(line.split(":")[1].split("(")[0].strip())
        elif "MCTS wins:" in line:
            mcts_wins = int(line.split(":")[1].split("(")[0].strip())
        elif "Draws:" in line:
            draws = int(line.split(":")[1].split("(")[0].strip())

    print(f"Results: NN={nn_wins}, MCTS={mcts_wins}, Draws={draws}")
    return nn_wins, mcts_wins, draws


def save_iteration(iteration: int):
    """Save current model state for this iteration."""
    iter_dir = ITERATIONS_DIR / f"iter_{iteration:03d}"
    iter_dir.mkdir(parents=True, exist_ok=True)

    for src in [MODEL_FINAL, ONNX_DRAFTING, ONNX_MOVEMENT]:
        if src.exists():
            shutil.copy(src, iter_dir / src.name)

    print(f"Saved iteration {iteration} to {iter_dir}")


def main():
    parser = argparse.ArgumentParser(description="Iterative HTMF training")
    parser.add_argument(
        "--iterations", type=int, default=20, help="Number of iterations"
    )
    parser.add_argument(
        "--games", type=int, default=200, help="Self-play games per iteration"
    )
    parser.add_argument(
        "--playouts", type=int, default=120_000, help="MCTS playouts per move"
    )
    parser.add_argument(
        "--epochs", type=int, default=20, help="Training epochs per iteration"
    )
    parser.add_argument(
        "--bootstrap", action="store_true", help="Start with traditional MCTS data"
    )
    parser.add_argument(
        "--fresh", action="store_true", help="Start fresh (delete existing model)"
    )
    args = parser.parse_args()

    ARTIFACTS_DIR.mkdir(parents=True, exist_ok=True)
    ITERATIONS_DIR.mkdir(parents=True, exist_ok=True)

    # Fresh start if requested
    if args.fresh:
        print("Starting fresh - removing existing model and data...")
        for f in [MODEL_FINAL, ONNX_DRAFTING, ONNX_MOVEMENT, TRAINING_DATA]:
            if f.exists():
                f.unlink()
                print(f"  Removed {f}")

    print("=" * 60)
    print("HTMF Iterative Training")
    print("=" * 60)
    print(f"Iterations: {args.iterations}")
    print(f"Games per iteration: {args.games}")
    print(f"Playouts per move: {args.playouts}")
    print(f"Epochs per iteration: {args.epochs}")
    print("=" * 60)

    # Check if we need to bootstrap
    if args.bootstrap or not MODEL_FINAL.exists():
        print("\nBootstrapping with traditional MCTS self-play...")
        # Generate high-quality data with many playouts
        new_data = generate_selfplay_data(args.games * 2, args.playouts, use_nn=False)
        merge_training_data(new_data)
        train_model(args.epochs * 2)  # Train longer for initial model
        save_iteration(0)

    for iteration in range(1, args.iterations + 1):
        print(f"\n{'=' * 60}")
        print(f"ITERATION {iteration}/{args.iterations}")
        print("=" * 60)

        # Generate self-play data using current NN
        new_data = generate_selfplay_data(args.games, args.playouts, use_nn=True)
        merge_training_data(new_data)

        # Train on all data
        train_model(args.epochs)

        # Save this iteration
        save_iteration(iteration)

        # Evaluate against pure MCTS
        nn_wins, mcts_wins, draws = evaluate_models()
        win_rate = nn_wins / max(1, nn_wins + mcts_wins + draws) * 100

        print(f"\nIteration {iteration} complete: NN win rate = {win_rate:.1f}%")

        if win_rate >= 55:
            print("Model is now competitive with pure MCTS!")

    print("\n" + "=" * 60)
    print("Training complete!")
    print("=" * 60)


if __name__ == "__main__":
    main()

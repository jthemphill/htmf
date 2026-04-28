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

import replay

# Paths
ARTIFACTS_DIR = Path("./artifacts")
TRAINING_DATA = ARTIFACTS_DIR / "training_data.jsonl"
MODEL_FINAL = ARTIFACTS_DIR / "model_final.pt"
ONNX_MODEL = ARTIFACTS_DIR / "model.onnx"
ITERATIONS_DIR = ARTIFACTS_DIR / "iterations"
BROWSER_MODEL = Path("../www/public/models/htmf-policy.onnx")


def run_command(cmd: list[str], cwd: str | None = None) -> subprocess.CompletedProcess:
    """Run a command and stream output."""
    print(f"$ {' '.join(cmd)}")
    result = subprocess.run(cmd, cwd=cwd)
    if result.returncode != 0:
        print(f"Command failed with exit code {result.returncode}")
    return result


def generate_selfplay_data(num_games: int, num_playouts: int, use_nn: bool) -> Path:
    """Generate self-play training data."""
    replay.SELFPLAY_DIR.mkdir(parents=True, exist_ok=True)
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    mode = "nn" if use_nn else "uniform"
    output_file = replay.SELFPLAY_DIR / (
        f"selfplay_{timestamp}_{mode}_g{num_games}_p{num_playouts}.jsonl"
    )

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
    """Rebuild the active training file from durable policy-v2 selfplay runs."""
    with open(new_data) as f:
        new_samples = sum(1 for _ in f)
    total_samples = replay.build_replay(max_samples=max_samples)
    print(f"Training data: {total_samples} samples (added {new_samples} new)")


def train_model(
    epochs: int,
    learning_rate: float = 0.001,
    num_filters: int | None = None,
    num_blocks: int | None = None,
) -> bool:
    """Train the model and return True if it improved."""
    print(f"\nTraining for {epochs} epochs...")
    cmd = [
        "uv",
        "run",
        "train.py",
        "--epochs",
        str(epochs),
        "--lr",
        str(learning_rate),
    ]
    if num_filters is not None:
        cmd.extend(["--num-filters", str(num_filters)])
    if num_blocks is not None:
        cmd.extend(["--num-blocks", str(num_blocks)])

    result = run_command(cmd, cwd=str(Path(__file__).parent))

    return result.returncode == 0


def evaluate_models(
    num_pairs: int = 100, num_playouts: int = 400, uniform_vs_uniform: bool = False
) -> tuple[int, int, int, float]:
    """
    Evaluate the model against the production uniform-prior baseline.
    Returns (model_wins, baseline_wins, draws, score)
    """
    print(f"\nEvaluating model ({num_pairs} pairs, {num_playouts} playouts)...")
    cmd = [
        "cargo",
        "run",
        "--release",
        "--bin",
        "nn_vs_mcts",
        "--",
        str(num_pairs),
        str(num_playouts),
    ]
    if uniform_vs_uniform:
        cmd.append("--uniform-vs-uniform")
    result = subprocess.run(
        cmd,
        capture_output=True,
        text=True,
        cwd=str(Path(__file__).parent.parent),
    )
    if result.returncode != 0:
        print(result.stdout)
        print(result.stderr)
        raise RuntimeError(f"evaluation failed with exit code {result.returncode}")

    # Parse results from output
    model_wins = 0
    baseline_wins = 0
    draws = 0
    score = 0.0

    for line in result.stdout.split("\n") + result.stderr.split("\n"):
        if "Model wins:" in line:
            model_wins = int(line.split(":")[1].split("(")[0].strip())
        elif "Baseline wins:" in line:
            baseline_wins = int(line.split(":")[1].split("(")[0].strip())
        elif "Draws:" in line:
            draws = int(line.split(":")[1].split("(")[0].strip())
        elif "Score:" in line:
            score = float(line.split(":")[1].strip())

    print(f"Results: model={model_wins}, baseline={baseline_wins}, draws={draws}, score={score:.3f}")
    return model_wins, baseline_wins, draws, score


def save_iteration(iteration: int):
    """Save current model state for this iteration."""
    iter_dir = ITERATIONS_DIR / f"iter_{iteration:03d}"
    iter_dir.mkdir(parents=True, exist_ok=True)

    for src in [MODEL_FINAL, ONNX_MODEL]:
        if src.exists():
            shutil.copy(src, iter_dir / src.name)

    print(f"Saved iteration {iteration} to {iter_dir}")


def promote_browser_model():
    """Copy the current ONNX model to the browser-served artifact path."""
    BROWSER_MODEL.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy(ONNX_MODEL, BROWSER_MODEL)
    print(f"Promoted browser model -> {BROWSER_MODEL}")


def backup_current_model() -> Path | None:
    """Save the current training artifacts so a failed iteration can roll back."""
    existing = [p for p in [MODEL_FINAL, ONNX_MODEL] if p.exists()]
    if not existing:
        return None

    backup_dir = ARTIFACTS_DIR / "rollback"
    if backup_dir.exists():
        shutil.rmtree(backup_dir)
    backup_dir.mkdir(parents=True)
    for src in existing:
        shutil.copy(src, backup_dir / src.name)
    return backup_dir


def restore_model_backup(backup_dir: Path | None):
    if backup_dir is None:
        return
    for src in backup_dir.iterdir():
        shutil.copy(src, ARTIFACTS_DIR / src.name)
    print(f"Restored previous model from {backup_dir}")


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
        "--eval-pairs", type=int, default=100, help="Paired evaluation seeds per iteration"
    )
    parser.add_argument(
        "--eval-playouts", type=int, default=400, help="Evaluation playouts per move"
    )
    parser.add_argument(
        "--promotion-score",
        type=float,
        default=0.53,
        help="Minimum model score vs uniform baseline required for promotion",
    )
    parser.add_argument(
        "--num-filters",
        type=int,
        default=None,
        help="Override number of convolutional filters; defaults to checkpoint metadata",
    )
    parser.add_argument(
        "--num-blocks",
        type=int,
        default=None,
        help="Override number of residual blocks; defaults to checkpoint metadata",
    )
    parser.add_argument(
        "--bootstrap", action="store_true", help="Start with uniform-prior baseline data"
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
        for f in [MODEL_FINAL, ONNX_MODEL, TRAINING_DATA]:
            if f.exists():
                f.unlink()
                print(f"  Removed {f}")
        if replay.SELFPLAY_DIR.exists():
            shutil.rmtree(replay.SELFPLAY_DIR)
            print(f"  Removed {replay.SELFPLAY_DIR}")

    print("=" * 60)
    print("HTMF Iterative Training")
    print("=" * 60)
    print(f"Iterations: {args.iterations}")
    print(f"Games per iteration: {args.games}")
    print(f"Playouts per move: {args.playouts}")
    print(f"Epochs per iteration: {args.epochs}")
    print(f"Eval pairs: {args.eval_pairs}")
    print(f"Eval playouts: {args.eval_playouts}")
    print(f"Promotion score: {args.promotion_score:.3f}")
    print("=" * 60)

    # Check if we need to bootstrap
    if args.bootstrap or not MODEL_FINAL.exists():
        print("\nBootstrapping with uniform-prior baseline self-play...")
        # Generate high-quality data with many playouts
        new_data = generate_selfplay_data(args.games * 2, args.playouts, use_nn=False)
        merge_training_data(new_data)
        train_model(args.epochs * 2, num_filters=args.num_filters, num_blocks=args.num_blocks)  # Train longer for initial model
        save_iteration(0)
        _, _, _, bootstrap_score = evaluate_models(args.eval_pairs, args.eval_playouts)
        if bootstrap_score >= args.promotion_score:
            promote_browser_model()

    for iteration in range(1, args.iterations + 1):
        print(f"\n{'=' * 60}")
        print(f"ITERATION {iteration}/{args.iterations}")
        print("=" * 60)

        # Generate self-play data using current NN
        new_data = generate_selfplay_data(args.games, args.playouts, use_nn=True)
        merge_training_data(new_data)

        backup_dir = backup_current_model()

        # Train on all data
        train_model(args.epochs, num_filters=args.num_filters, num_blocks=args.num_blocks)

        # Save this iteration
        save_iteration(iteration)

        # Evaluate against the production uniform-prior baseline
        model_wins, baseline_wins, draws, score = evaluate_models(
            args.eval_pairs, args.eval_playouts
        )

        print(
            f"\nIteration {iteration} complete: model={model_wins}, baseline={baseline_wins}, "
            f"draws={draws}, score={score:.3f}"
        )

        if score >= args.promotion_score:
            print("Model passed promotion gate.")
            promote_browser_model()
        else:
            print("Model did not pass promotion gate; rolling back training artifact.")
            restore_model_backup(backup_dir)

    print("\n" + "=" * 60)
    print("Training complete!")
    print("=" * 60)


if __name__ == "__main__":
    main()

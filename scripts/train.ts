#!/usr/bin/env bun
/**
 * HTMF Training Script
 *
 * Runs one iteration of selfplay and training, then evaluates the new model
 * against the baseline (uniform prior). If the new model is at least as strong,
 * it's promoted; otherwise rolled back.
 *
 * Key insight: Only use NN-guided selfplay once the model has "graduated"
 * (beaten the uniform prior). Until then, use traditional MCTS selfplay
 * to generate high-quality training data.
 */

import { spawn } from "bun";
import { existsSync, mkdirSync, copyFileSync, unlinkSync, renameSync, writeFileSync } from "fs";
import { join } from "path";

// Configuration
const SELFPLAY_GAMES = 50;
const SELFPLAY_PLAYOUTS = 1000;
const TRAIN_EPOCHS = 10;
const EVAL_GAMES = 20;
const EVAL_PLAYOUTS = 200;

// Paths (relative to project root)
const ARTIFACTS_DIR = "training/artifacts";
const MODEL_DRAFTING = join(ARTIFACTS_DIR, "model_drafting.onnx");
const MODEL_MOVEMENT = join(ARTIFACTS_DIR, "model_movement.onnx");
const MODEL_FINAL = join(ARTIFACTS_DIR, "model_final.pt");
const PREV_DRAFTING = join(ARTIFACTS_DIR, "prev_model_drafting.onnx");
const PREV_MOVEMENT = join(ARTIFACTS_DIR, "prev_model_movement.onnx");
const PREV_FINAL = join(ARTIFACTS_DIR, "prev_model_final.pt");
const BLANK_DRAFTING = join(ARTIFACTS_DIR, "blank_model_drafting.onnx");
const BLANK_MOVEMENT = join(ARTIFACTS_DIR, "blank_model_movement.onnx");
const TRAINING_DATA = join(ARTIFACTS_DIR, "training_data.jsonl");
// Marker file indicating the model has graduated (beaten the baseline)
const GRADUATED_MARKER = join(ARTIFACTS_DIR, ".graduated");

async function run(cmd: string[], options?: { cwd?: string; stdout?: "pipe" | "inherit" }): Promise<{ exitCode: number; stdout?: string }> {
  const proc = spawn({
    cmd,
    cwd: options?.cwd,
    stdout: options?.stdout ?? "inherit",
    stderr: "inherit",
  });

  const exitCode = await proc.exited;

  if (options?.stdout === "pipe") {
    const stdout = await new Response(proc.stdout).text();
    return { exitCode, stdout };
  }

  return { exitCode };
}

async function runWithOutput(cmd: string[], cwd?: string): Promise<string> {
  const proc = spawn({
    cmd,
    cwd,
    stdout: "pipe",
    stderr: "pipe",
  });

  const [stdout, stderr] = await Promise.all([
    new Response(proc.stdout).text(),
    new Response(proc.stderr).text(),
  ]);

  await proc.exited;
  return stdout + stderr;
}

function countLines(filePath: string): number {
  const file = Bun.file(filePath);
  const text = file.size > 0 ? require("fs").readFileSync(filePath, "utf-8") : "";
  return text.split("\n").filter((line: string) => line.trim()).length;
}

async function mergeTrainingData(selfplayFile: string, maxSamples = 100000): Promise<number> {
  const newData = await Bun.file(selfplayFile).text();
  const newLines = newData.split("\n").filter(line => line.trim());

  let allLines: string[];

  if (existsSync(TRAINING_DATA)) {
    const existingData = await Bun.file(TRAINING_DATA).text();
    const existingLines = existingData.split("\n").filter(line => line.trim());
    allLines = [...existingLines, ...newLines];
  } else {
    allLines = newLines;
  }

  // Keep only the most recent samples
  if (allLines.length > maxSamples) {
    allLines = allLines.slice(-maxSamples);
  }

  await Bun.write(TRAINING_DATA, allLines.join("\n") + "\n");
  return allLines.length;
}

function parseEvalResults(output: string): { nnWins: number; mctsWins: number; draws: number } {
  const nnMatch = output.match(/NN wins:\s*(\d+)/);
  const mctsMatch = output.match(/MCTS wins:\s*(\d+)/);
  const drawsMatch = output.match(/Draws:\s*(\d+)/);

  return {
    nnWins: nnMatch ? parseInt(nnMatch[1], 10) : 0,
    mctsWins: mctsMatch ? parseInt(mctsMatch[1], 10) : 0,
    draws: drawsMatch ? parseInt(drawsMatch[1], 10) : 0,
  };
}

async function main(): Promise<number> {
  console.log("==============================================");
  console.log("HTMF Training Iteration");
  console.log("==============================================");

  // Ensure artifacts directory exists
  if (!existsSync(ARTIFACTS_DIR)) {
    mkdirSync(ARTIFACTS_DIR, { recursive: true });
  }

  // Determine if this is the first training run (no model exists)
  const isFirstRun = !existsSync(MODEL_DRAFTING) || !existsSync(MODEL_MOVEMENT);

  if (isFirstRun) {
    console.log("First training run detected - will compare against uniform prior");

    // Ensure blank models exist
    if (!existsSync(BLANK_DRAFTING) || !existsSync(BLANK_MOVEMENT)) {
      console.log("Creating blank models for uniform prior baseline...");
      const result = await run(["uv", "run", "create_blank_models.py"], { cwd: "training" });
      if (result.exitCode !== 0) {
        console.error("Failed to create blank models");
        return 1;
      }
    }
  } else {
    // Save current model as "previous" (for comparison after training)
    console.log("\nBacking up current model...");
    copyFileSync(MODEL_DRAFTING, PREV_DRAFTING);
    copyFileSync(MODEL_MOVEMENT, PREV_MOVEMENT);
    copyFileSync(MODEL_FINAL, PREV_FINAL);
  }

  // Check if model has graduated (beaten baseline before)
  const hasGraduated = existsSync(GRADUATED_MARKER);

  // Step 1: Generate selfplay data
  // Only use NN-guided selfplay if the model has graduated (beaten baseline)
  // Otherwise, use traditional MCTS to generate high-quality training data
  const useNn = hasGraduated;
  const selfplayMode = useNn ? "NN-guided MCTS" : "traditional MCTS (uniform prior)";
  console.log(`\nStep 1: Generating ${SELFPLAY_GAMES} selfplay games (${SELFPLAY_PLAYOUTS} playouts/move)...`);
  console.log(`Using ${selfplayMode} for selfplay`);

  const timestamp = new Date().toISOString().replace(/[-:T.]/g, "").slice(0, 15);
  const selfplayFile = join(ARTIFACTS_DIR, `selfplay_${timestamp}.jsonl`);

  const selfplayArgs = ["cargo", "run", "--release", "-p", "selfplay", "--", String(SELFPLAY_GAMES), String(SELFPLAY_PLAYOUTS)];
  if (useNn) {
    selfplayArgs.push("--nn");
  }

  const selfplayProc = spawn({
    cmd: selfplayArgs,
    stdout: "pipe",
    stderr: "inherit",
  });

  const selfplayOutput = await new Response(selfplayProc.stdout).text();
  await Bun.write(selfplayFile, selfplayOutput);

  const selfplayExit = await selfplayProc.exited;
  if (selfplayExit !== 0) {
    console.error("Selfplay failed");
    return 1;
  }

  const numSamples = countLines(selfplayFile);
  console.log(`Generated ${numSamples} training samples`);

  // Merge with existing training data
  const totalSamples = await mergeTrainingData(selfplayFile);
  console.log(`Total training samples: ${totalSamples}`);

  // Clean up selfplay file
  unlinkSync(selfplayFile);

  // Step 2: Train the model
  console.log(`\nStep 2: Training for ${TRAIN_EPOCHS} epochs...`);
  const trainResult = await run(["uv", "run", "train.py", "--epochs", String(TRAIN_EPOCHS)], { cwd: "training" });
  if (trainResult.exitCode !== 0) {
    console.error("Training failed");
    return 1;
  }

  // Step 3: Evaluate new model vs baseline
  console.log(`\nStep 3: Evaluating new model (${EVAL_GAMES} games, ${EVAL_PLAYOUTS} playouts/move)...`);

  const evalOutput = await runWithOutput([
    "cargo", "run", "--release", "--bin", "nn_vs_mcts", "--",
    String(EVAL_GAMES), String(EVAL_PLAYOUTS)
  ]);

  console.log(evalOutput);

  const { nnWins, mctsWins, draws } = parseEvalResults(evalOutput);

  console.log("\n==============================================");
  console.log(`Results: NN=${nnWins}, MCTS=${mctsWins}, Draws=${draws}`);

  // Determine if new model is at least as strong
  if (nnWins >= mctsWins) {
    console.log("New model is at least as strong as baseline!");
    console.log("Model promoted successfully.");

    // Mark model as graduated (can now use NN-guided selfplay)
    if (!hasGraduated) {
      writeFileSync(GRADUATED_MARKER, new Date().toISOString());
      console.log("Model has graduated! Future selfplay will use NN-guided MCTS.");
    }

    // Clean up previous model backup
    if (existsSync(PREV_DRAFTING)) unlinkSync(PREV_DRAFTING);
    if (existsSync(PREV_MOVEMENT)) unlinkSync(PREV_MOVEMENT);
    if (existsSync(PREV_FINAL)) unlinkSync(PREV_FINAL);

    console.log("==============================================");
    return 0;
  } else {
    console.log("New model is weaker than baseline.");

    if (isFirstRun) {
      console.log("First model not strong enough yet - keeping it for next iteration.");
      console.log("(More training data may help)");
    } else {
      console.log("Rolling back to previous model...");
      renameSync(PREV_DRAFTING, MODEL_DRAFTING);
      renameSync(PREV_MOVEMENT, MODEL_MOVEMENT);
      renameSync(PREV_FINAL, MODEL_FINAL);
      console.log("Previous model restored.");
    }

    console.log("==============================================");
    return 1;
  }
}

const exitCode = await main();
process.exit(exitCode);

import * as wasm from "htmf-wasm";

import {
  NPLAYERS,
  NUM_CELLS,
  MIN_PLAYOUTS,
  MAX_PLAYOUTS,
  HUMAN_PLAYER,
  BOT_PLAYER,
  PLAYOUT_CHUNK_SIZE,
} from "./constants";
import {
  type GameState,
  type MoveScore,
  type WorkerRequest,
  type WorkerResponse,
} from "./WorkerProtocol";

function getGameState(game: wasm.Game): GameState {
  const fish = [];
  for (let idx = 0; idx < NUM_CELLS; ++idx) {
    fish.push(game.num_fish(idx));
  }
  const scores: number[] = [];
  const penguins = [];
  const claimed = [];
  for (let p = 0; p < NPLAYERS; ++p) {
    scores.push(game.score(p));
    penguins.push([...game.penguins(p)]);
    claimed.push([...game.claimed(p)]);
  }
  return {
    activePlayer: game.active_player(),
    modeType: game.finished_drafting() ? "playing" : "drafting",
    scores,
    turn: game.turn(),
    board: {
      fish,
      penguins,
      claimed,
    },
  };
}

function getPossibleMoves(game: wasm.Game, src?: number): number[] {
  if (game.active_player() !== HUMAN_PLAYER) {
    return [];
  }
  if (game.finished_drafting()) {
    if (src !== undefined) {
      return [...game.possible_moves(src)];
    }
    const activePlayer = game.active_player();
    return activePlayer === undefined ? [] : [...game.penguins(activePlayer)];
  } else {
    return [...game.draftable_cells()];
  }
}

class Bot {
  wasmInternals: wasm.InitOutput;
  game: wasm.Game = wasm.Game.new();
  postMessage: (msg: WorkerResponse) => void;
  ponderer?: number;
  ponderStartTime?: number;
  totalCompletedPonderTimeMs: number = 0;

  constructor(
    wasmInternals: wasm.InitOutput,
    postMessage: (msg: WorkerResponse) => void,
  ) {
    this.wasmInternals = wasmInternals;
    this.postMessage = postMessage;
    this.ponder();
    this.postGameState({});
  }

  free(): void {
    this.stopPondering();
    this.game.free();
    this.totalCompletedPonderTimeMs = 0;
  }

  ponder(): void {
    if (this.ponderer !== undefined) {
      return;
    }
    this.ponderStartTime = performance.now();
    this.ponderer = self.setInterval(() => {
      const activePlayer = this.game.active_player();
      if (
        this.game.get_visits() >= MAX_PLAYOUTS ||
        activePlayer === BOT_PLAYER
      ) {
        this.stopPondering();
        return;
      }

      this.game.playout_n_times(PLAYOUT_CHUNK_SIZE);

      if (activePlayer !== undefined) {
        this.postThinkingProgress({
          activePlayer,
          playoutsNeeded: MAX_PLAYOUTS,
        });
      }
    });
  }

  stopPondering(): void {
    if (this.ponderer !== undefined) {
      clearInterval(this.ponderer);
      this.ponderer = undefined;
    }
    if (this.ponderStartTime !== undefined) {
      this.totalCompletedPonderTimeMs +=
        performance.now() - this.ponderStartTime;
      this.ponderStartTime = undefined;
    }
  }

  placePenguin(dst: number): void {
    this.game.place_penguin(dst);
    this.ponder();
  }

  movePenguin(src: number, dst: number): void {
    this.game.move_penguin(src, dst);
    this.ponder();
  }

  playout(): void {
    this.game.playout();
  }

  takeAction(): void {
    const playoutsNeeded =
      this.game.turn() < 2 ? 2 * MIN_PLAYOUTS : MIN_PLAYOUTS;
    let lastIntervalTime = performance.now();
    while (this.game.get_visits() < playoutsNeeded) {
      this.game.playout_n_times(PLAYOUT_CHUNK_SIZE);
      this.totalCompletedPonderTimeMs += performance.now() - lastIntervalTime;
      lastIntervalTime = performance.now();

      const activePlayer = this.game.active_player();
      if (activePlayer !== undefined) {
        this.postThinkingProgress({ activePlayer, playoutsNeeded });
      }
    }
    this.totalCompletedPonderTimeMs += performance.now() - lastIntervalTime;
    lastIntervalTime = performance.now();

    this.game.take_action();
    this.postGameState({});
    this.ponder();
  }

  getState(): GameState {
    return getGameState(this.game);
  }

  getPossibleMoves(src?: number): number[] {
    return getPossibleMoves(this.game, src);
  }

  onMessage(request: WorkerRequest): void {
    console.log(`received request ${request.type}`);
    let src;
    let lastMoveWasIllegal = false;
    switch (request.type) {
      case "getGameState":
        break;
      case "movePenguin":
        try {
          if (request.src === undefined) {
            this.placePenguin(request.dst);
          } else {
            this.movePenguin(request.src, request.dst);
          }
        } catch (err) {
          lastMoveWasIllegal = true;
        }
        break;
      case "getPossibleMoves":
        src = request.src;
        break;
    }
    this.postGameState({ src, lastMoveWasIllegal });
    while (this.game.active_player() === BOT_PLAYER) {
      this.takeAction();
    }
  }

  postGameState({
    src,
    lastMoveWasIllegal,
  }: {
    src?: number;
    lastMoveWasIllegal?: boolean;
  }): void {
    lastMoveWasIllegal = lastMoveWasIllegal === true;
    const postMessage = this.postMessage;
    postMessage({
      type: "gameState",
      gameState: this.getState(),
      possibleMoves: this.getPossibleMoves(src),
      lastMoveWasIllegal,
    });
  }

  postThinkingProgress({
    activePlayer,
    playoutsNeeded,
  }: {
    activePlayer: number;
    playoutsNeeded: number;
  }): void {
    const postMessage = this.postMessage;
    postMessage({
      type: "thinkingProgress",
      completed: this.game.get_visits(),
      required: playoutsNeeded,
      totalPlayouts: this.game.get_total_playouts(),
      totalTimeThinkingMs: this.getTotalTimeThinkingMs(),
      memoryUsage: this.wasmInternals.memory.buffer.byteLength,
      treeSize: this.game.tree_size(),
      playerMoveScores: {
        player: activePlayer,
        moveScores: this.getMoveScores(activePlayer),
      },
    });
  }

  getMoveScores(activePlayer: number): MoveScore[] {
    const moveScores: MoveScore[] = [];
    if (this.game.is_drafting()) {
      for (const dst of this.game.draftable_cells()) {
        const info = this.game.place_info(dst);
        moveScores.push({
          dst,
          visits: info.get_visits(),
          rewards: info.get_rewards(),
        });
      }
    } else {
      for (const src of this.game.penguins(activePlayer)) {
        for (const dst of this.game.possible_moves(src)) {
          const info = this.game.move_info(src, dst);
          moveScores.push({
            src,
            dst,
            visits: info.get_visits(),
            rewards: info.get_rewards(),
          });
        }
      }
    }
    return moveScores;
  }

  getTotalTimeThinkingMs(): number {
    let totalTimeThinkingMs = this.totalCompletedPonderTimeMs;
    if (this.ponderStartTime !== undefined) {
      totalTimeThinkingMs += performance.now() - this.ponderStartTime;
    }
    return totalTimeThinkingMs;
  }
}

export default Bot;

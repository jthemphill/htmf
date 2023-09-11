import * as wasm from '../../pkg/htmf_wasm'

import { NPLAYERS, NUM_CELLS, PONDER_INTERVAL_MS, MIN_PLAYOUTS, MAX_PLAYOUTS, HUMAN_PLAYER, BOT_PLAYER } from './constants'
import { type GameState, type MoveScore, type WorkerRequest, type WorkerResponse } from './WorkerProtocol'

function getGameState (game: wasm.Game): GameState {
  const fish = []
  for (let idx = 0; idx < NUM_CELLS; ++idx) {
    fish.push(game.num_fish(idx))
  }
  const scores: number[] = []
  const penguins = []
  const claimed = []
  for (let p = 0; p < NPLAYERS; ++p) {
    scores.push(game.score(p))
    penguins.push([...game.penguins(p)])
    claimed.push([...game.claimed(p)])
  }
  return {
    activePlayer: game.active_player(),
    modeType: game.finished_drafting() ? 'playing' : 'drafting',
    scores,
    turn: game.turn(),
    board: {
      fish,
      penguins,
      claimed
    }
  }
}

function getPossibleMoves (game: wasm.Game, src?: number): number[] {
  if (game.active_player() !== HUMAN_PLAYER) {
    return []
  }
  if (game.finished_drafting()) {
    if (src !== undefined) {
      return [...game.possible_moves(src)]
    }
    const activePlayer = game.active_player()
    return activePlayer === undefined ? [] : [...game.penguins(activePlayer)]
  } else {
    return [...game.draftable_cells()]
  }
}

class Bot {
  wasmInternals: wasm.InitOutput
  game: wasm.Game
  postMessage: (msg: WorkerResponse) => void
  ponderer?: number
  nplayouts = 0

  constructor (wasmInternals: wasm.InitOutput, postMessage: (msg: WorkerResponse) => void) {
    this.wasmInternals = wasmInternals
    this.postMessage = postMessage
    this.game = wasm.Game.new()
    this.ponder()
  }

  free (): void {
    this.stopPondering()
    this.game.free()
  }

  reset (): void {
    this.free()
    this.game = wasm.Game.new()
    this.ponder()
    this.postGameState({ })
  }

  ponder (): void {
    this.nplayouts = this.game.get_visits()

    if (this.ponderer !== undefined) {
      return
    }
    const ponderStartTime = performance.now()
    this.ponderer = self.setInterval(() => {
      if (this.nplayouts >= MAX_PLAYOUTS) {
        this.stopPondering()
        return
      }
      const intervalStartTime = performance.now()
      while (performance.now() - intervalStartTime < PONDER_INTERVAL_MS) {
        this.playout()
      }

      const activePlayer = this.game.active_player()
      if (activePlayer !== undefined) {
        const totalTimeMs = performance.now() - ponderStartTime
        this.postThinkingProgress({ activePlayer, playoutsNeeded: MAX_PLAYOUTS, totalTimeMs })
      }
    })
  }

  stopPondering (): void {
    if (this.ponderer !== undefined) {
      clearInterval(this.ponderer)
      this.ponderer = undefined
    }
  }

  placePenguin (dst: number): void {
    this.game.place_penguin(dst)
    this.ponder()
  }

  movePenguin (src: number, dst: number): void {
    this.game.move_penguin(src, dst)
    this.ponder()
  }

  playout (): void {
    this.game.playout()
    ++this.nplayouts
  }

  takeAction (): void {
    this.stopPondering()
    const playoutsNeeded = this.game.turn() < 2 ? 2 * MIN_PLAYOUTS : MIN_PLAYOUTS
    const startTime = performance.now()
    while (this.nplayouts < playoutsNeeded) {
      this.playout()
      if (this.nplayouts % 100 === 0) {
        const activePlayer = this.game.active_player()
        if (activePlayer !== undefined) {
          const totalTimeMs = performance.now() - startTime
          this.postThinkingProgress({ activePlayer, playoutsNeeded, totalTimeMs })
        }
      }
    }
    this.game.take_action()
    this.postGameState({ })
    this.ponder()
  }

  getState (): GameState {
    return getGameState(this.game)
  }

  getPossibleMoves (src?: number): number[] {
    return getPossibleMoves(this.game, src)
  }

  onMessage (request: WorkerRequest): void {
    console.log(`received request ${request.type}`)
    let src
    let lastMoveWasIllegal = false
    switch (request.type) {
      case 'getGameState':
        break
      case 'movePenguin':
        try {
          if (request.src === undefined) {
            this.placePenguin(request.dst)
          } else {
            this.movePenguin(request.src, request.dst)
          }
        } catch (err) {
          lastMoveWasIllegal = true
        }
        break
      case 'getPossibleMoves':
        src = request.src
        break
    }
    this.postGameState({ src, lastMoveWasIllegal })
    while (this.game.active_player() === BOT_PLAYER) {
      this.takeAction()
    }
  }

  postGameState ({ src, lastMoveWasIllegal }: { src?: number, lastMoveWasIllegal?: boolean }): void {
    lastMoveWasIllegal = lastMoveWasIllegal === true
    const postMessage = this.postMessage
    postMessage({
      type: 'gameState',
      gameState: this.getState(),
      possibleMoves: this.getPossibleMoves(src),
      lastMoveWasIllegal
    })
  }

  postThinkingProgress ({
    activePlayer,
    playoutsNeeded,
    totalTimeMs
  }: { activePlayer: number, playoutsNeeded: number, totalTimeMs: number }): void {
    const postMessage = this.postMessage
    postMessage({
      type: 'thinkingProgress',
      completed: this.nplayouts,
      required: playoutsNeeded,
      totalTimeMs,
      memoryUsage: this.wasmInternals.memory.buffer.byteLength,
      treeSize: this.game.tree_size(),
      playerMoveScores: { player: activePlayer, moveScores: this.getMoveScores(activePlayer) }
    })
  }

  getMoveScores (activePlayer: number): MoveScore[] {
    const moveScores: MoveScore[] = []
    if (this.game.is_drafting()) {
      for (const dst of this.game.draftable_cells()) {
        const info = this.game.place_info(dst)
        moveScores.push({
          dst,
          visits: info.get_visits(),
          rewards: info.get_rewards()
        })
      }
    } else {
      for (const src of this.game.penguins(activePlayer)) {
        for (const dst of this.game.possible_moves(src)) {
          const info = this.game.move_info(src, dst)
          moveScores.push({
            src,
            dst,
            visits: info.get_visits(),
            rewards: info.get_rewards()
          })
        }
      }
    }
    return moveScores
  }
}

export default Bot

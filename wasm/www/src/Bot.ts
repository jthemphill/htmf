import * as wasm from 'htmf-wasm'

import { NPLAYERS, NUM_CELLS, PLAYOUT_MS, PONDER_INTERVAL_MS, MIN_PLAYOUTS, MAX_PLAYOUTS, HUMAN_PLAYER, BOT_PLAYER } from './constants'
import type GameState from './GameState'
import { type WorkerRequest, type WorkerResponse } from './WorkerProtocol'

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
    this.game = wasm.Game.new()
    this.postMessage = postMessage
    this.ponder()
  }

  free (): void {
    this.stopPondering()
    this.game.free()
  }

  init (): void {
    const postMessage = this.postMessage
    postMessage({
      type: 'initialized',
      gameState: this.getState(),
      possibleMoves: this.getPossibleMoves()
    })
  }

  ponder (): void {
    this.nplayouts = this.game.get_visits()

    if (this.ponderer !== undefined) {
      return
    }
    this.ponderer = setInterval(
      () => {
        if (this.nplayouts >= MAX_PLAYOUTS) {
          this.stopPondering()
          return
        }
        const t0 = performance.now()
        while (performance.now() - t0 < PLAYOUT_MS) {
          this.playout()
        }

        const activePlayer = this.game.active_player()
        if (activePlayer !== undefined) {
          if (this.game.is_drafting()) {
            this.postPlaceScores(activePlayer)
          } else {
            this.postMoveScores(activePlayer)
          }
        }
      },
      PONDER_INTERVAL_MS
    )
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
    const postMessage = this.postMessage
    const minPlayouts = this.game.turn() < 2 ? 2 * MIN_PLAYOUTS : MIN_PLAYOUTS
    const startTime = performance.now()
    while (this.nplayouts < minPlayouts) {
      this.playout()
      if (this.nplayouts % 100 === 0) {
        postMessage({
          type: 'thinkingProgress',
          completed: this.nplayouts,
          required: minPlayouts,
          totalTimeMs: performance.now() - startTime
        })
        if (this.game.is_drafting()) {
          this.postPlaceScores(BOT_PLAYER)
        } else {
          this.postMoveScores(BOT_PLAYER)
        }
      }
    }
    this.game.take_action()
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
    switch (request.type) {
      case 'get':
        this.postGameState()
        break
      case 'move':
        try {
          this.movePenguin(request.src, request.dst)
          this.postGameState()
        } catch (err) {
          this.postIllegalMove(request.src, request.dst)
        }
        break
      case 'place':
        try {
          this.placePenguin(request.dst)
          this.postGameState()
        } catch (err) {
          this.postIllegalPlacement(request.dst)
        }
        break
      case 'possibleMoves':
        this.postPossibleMoves(request.src)
        break
      case 'takeAction':
        this.takeAction()
        this.postGameState()
        break
    }
  }

  postIllegalMove (src: number, dst: number): void {
    const postMessage = this.postMessage
    postMessage({
      type: 'illegalMove',
      src,
      dst
    })
  }

  postIllegalPlacement (dst: number): void {
    const postMessage = this.postMessage
    postMessage({
      type: 'illegalPlacement',
      dst
    })
  }

  postPossibleMoves (src?: number): void {
    const postMessage = this.postMessage
    postMessage({
      type: 'possibleMoves',
      possibleMoves: this.getPossibleMoves(src)
    })
  }

  postGameState (): void {
    const postMessage = this.postMessage
    postMessage({
      type: 'state',
      gameState: this.getState()
    })
  }

  postPlaceScores (activePlayer: number): void {
    const placeScores = []
    for (const dst of this.game.draftable_cells()) {
      const info = this.game.place_info(dst)
      placeScores.push({
        dst,
        visits: info.get_visits(),
        rewards: info.get_rewards()
      })
    }
    const postMessage = this.postMessage
    postMessage({
      type: 'placeScores',
      activePlayer,
      placeScores,
      memoryUsage: this.wasmInternals.memory.buffer.byteLength,
      treeSize: this.game.tree_size()
    })
  }

  postMoveScores (activePlayer: number): void {
    const moveScores = []
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
    const postMessage = this.postMessage
    postMessage({
      type: 'moveScores',
      activePlayer,
      moveScores,
      memoryUsage: this.wasmInternals.memory.buffer.byteLength,
      treeSize: this.game.tree_size()
    })
  }
}

export default Bot

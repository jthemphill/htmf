import type GameState from './GameState'

export type WorkerRequest = {
  type: 'get'
} | {
  type: 'place'
  dst: number
} | {
  type: 'move'
  src: number
  dst: number
} | {
  type: 'possibleMoves'
  src?: number
} | {
  type: 'takeAction'
} | {
  type: 'startPondering'
} | {
  type: 'takeAction'
}

export type WorkerResponse = {
  type: 'initialized'
  gameState: GameState
  possibleMoves: number[]
} | {
  type: 'state'
  gameState: GameState
} | {
  type: 'possibleMoves'
  possibleMoves: number[]
} | {
  type: 'illegalMove'
  src: number
  dst: number
} | {
  type: 'illegalPlacement'
  dst: number
} | {
  type: 'placeScores'
  activePlayer: number
  placeScores: Array<{
    dst: number
    visits: number
    rewards: number
  }>
} | {
  type: 'moveScores'
  activePlayer: number
  moveScores: Array<{
    src: number
    dst: number
    visits: number
    rewards: number
  }>
} | {
  type: 'thinkingProgress'
  completed: number
  required: number
  totalTimeMs: number
}

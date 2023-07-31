export default interface GameState {
  activePlayer?: number
  modeType: 'drafting' | 'playing'
  scores: number[]
  turn: number
  board: {
    fish: number[]
    penguins: number[][]
    claimed: number[][]
  }
}

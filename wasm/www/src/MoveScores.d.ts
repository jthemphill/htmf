interface MoveScore { src?: number, dst: number, visits: number, rewards: number }

export interface MoveScores {
  player: number
  tally: MoveScore[]
}

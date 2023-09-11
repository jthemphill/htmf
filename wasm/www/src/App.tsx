import React from 'react'

import Board from './Board'
import { BOT_PLAYER, HUMAN_PLAYER, NPLAYERS } from './constants'
import { type GameState, type PlayerMoveScores, type WorkerRequest, type WorkerResponse } from './WorkerProtocol'

interface ThinkingProgress {
  completed: number
  required: number
  totalPlayouts: number
  totalTimeMs: number
}

export interface AppWorker extends Worker {
  onmessage: (event: MessageEvent<WorkerResponse>) => void
  postMessage: (request: WorkerRequest) => void
}

interface Props {
  worker: AppWorker
}

export default function App ({ worker }: Props): React.JSX.Element {
  const [gameState, setGameState] = React.useState<GameState | undefined>(undefined)
  const [possibleMoves, setPossibleMoves] = React.useState<number[] | undefined>(undefined)
  const [chosenCell, setChosenCell] = React.useState<number | undefined>(undefined)
  const [lastMoveWasIllegal, setLastMoveWasIllegal] = React.useState<boolean>(false)
  const [playerMoveScores, setPlayerMoveScores] = React.useState<PlayerMoveScores | undefined>(undefined)
  const [thinkingProgress, setThinkingProgress] = React.useState<ThinkingProgress | undefined>(undefined)
  const [treeSize, setTreeSize] = React.useState<number>(0)
  const [memoryUsage, setMemoryUsage] = React.useState<number>(0)

  React.useEffect(() => {
    worker.onmessage = ({ data: response }: MessageEvent<WorkerResponse>) => {
      switch (response.type) {
        case 'gameState':
          setGameState(response.gameState)
          setPossibleMoves(response.possibleMoves)
          setLastMoveWasIllegal(response.lastMoveWasIllegal)
          break
        case 'thinkingProgress':
          setPlayerMoveScores(response.playerMoveScores)
          setTreeSize(response.treeSize)
          setMemoryUsage(response.memoryUsage)
          setThinkingProgress({
            completed: response.completed,
            required: response.required,
            totalPlayouts: response.totalPlayouts,
            totalTimeMs: response.totalTimeThinkingMs
          })
          break
      }
    }
    worker.postMessage({ type: 'getGameState' })
  }, [
    worker,
    setGameState,
    setPossibleMoves,
    setChosenCell,
    setLastMoveWasIllegal,
    setPlayerMoveScores,
    setThinkingProgress,
    setTreeSize,
    setMemoryUsage
  ])

  const modeType = gameState?.modeType
  const handleCellClick = React.useCallback((key: number) => {
    // Reset state if we know it's not a legal move
    if (modeType === undefined || possibleMoves?.includes(key) !== true) {
      setChosenCell(undefined)
      worker.postMessage({ type: 'getPossibleMoves' })
      return
    }

    // Place a penguin if we're still drafting
    if (modeType === 'drafting') {
      worker.postMessage({
        type: 'movePenguin',
        dst: key
      })
      return
    }

    // Choose the given cell to move a penguin from
    if (chosenCell === undefined) {
      setChosenCell(key)
      worker.postMessage({ type: 'getPossibleMoves', src: key })
      return
    }

    // If we have already chosen a penguin, move the penguin to the clicked cell
    worker.postMessage({
      type: 'movePenguin',
      src: chosenCell,
      dst: key
    })
    setChosenCell(undefined)
  }, [worker, modeType, possibleMoves, chosenCell])

  let topMove
  if (playerMoveScores !== undefined && playerMoveScores.player === BOT_PLAYER) {
    for (const score of playerMoveScores.moveScores) {
      if (topMove === undefined || score.rewards > topMove.rewards) {
        topMove = score
      }
    }
  }

  return (
    <div className="app">
      {
        gameState !== undefined &&
        <Board
          gameState={gameState}
          possibleMoves={possibleMoves ?? []}
          chosenCell={chosenCell}
          handleCellClick={handleCellClick}
          topMove={topMove}
        />
      }
      <div className="info-col">
        <p>{gameState?.modeType}</p>
        {lastMoveWasIllegal && <p>"Invalid move!"</p>}
        {gameState !== undefined && <ScoresBlock activePlayer={gameState.activePlayer} scores={gameState.scores} />}
        {playerMoveScores !== undefined && <WinChanceMeter playerMoveScores={playerMoveScores} />}
        <p className="tree-size">Stored {treeSize.toLocaleString()} game states in memory</p>
        <MemoryUsageBlock memoryUsage={memoryUsage} treeSize={treeSize} />
        {thinkingProgress !== undefined && <ThinkingProgressBar thinkingProgress={thinkingProgress} />}
      </div>
    </div>
  )
}

const ScoresBlock = React.memo(
  function ScoresBlock ({ activePlayer, scores }: { activePlayer?: number, scores: number[] }): React.JSX.Element {
    const scoresBlock = []
    for (let p = 0; p < NPLAYERS; ++p) {
      const playerClass = p === HUMAN_PLAYER ? 'human' : 'bot'
      const active = activePlayer === p ? '(Active Player)' : undefined
      scoresBlock.push(
        <p key={'score_' + p.toString()}><span className={playerClass}>
          Score: {scores[p]} {active}
        </span></p>
      )
    }
    return <div className="scores-block">{scoresBlock}</div>
  }
)

const WinChanceMeter = React.memo(
  function WinChanceMeter ({ playerMoveScores }: { playerMoveScores: PlayerMoveScores }): React.JSX.Element {
    let totalVisits = 0
    let totalRewards = 0
    for (const mov of playerMoveScores.moveScores) {
      totalVisits += mov.visits
      totalRewards += mov.rewards
    }

    let chance = totalVisits > 0 ? totalRewards / totalVisits : 0.5
    if (playerMoveScores.player !== HUMAN_PLAYER) {
      chance = 1 - chance
    }

    return (
      <div className="win-chance">
        <meter id="win-chance-bar" min={0} max={1} low={0.49} high={0.5} optimum={1} value={chance} />
        <label htmlFor="win-chance-bar">Who's winning? (higher is better for you)</label>
      </div>
    )
  }
)

const MemoryUsageBlock = React.memo(
  function MemoryUsageBlock ({ memoryUsage, treeSize }: { memoryUsage: number, treeSize: number }): React.JSX.Element {
    const memoryUsageFormatter = new Intl.NumberFormat(
      navigator.language,
      { notation: 'compact', style: 'unit', unit: 'byte', unitDisplay: 'narrow' }
    )
    return (
      <div className="memory-usage-block">
        <p className="memory-usage">Using {memoryUsageFormatter.format(memoryUsage)}</p>
        <p className="memory-ratio">Ratio is {memoryUsageFormatter.format(memoryUsage / treeSize)} per state</p>
      </div>
    )
  }
)

const ThinkingProgressBar = React.memo(
  function ThinkingProgressBar ({ thinkingProgress }: { thinkingProgress: ThinkingProgress }): React.JSX.Element {
    const playoutsPerSec = thinkingProgress.totalPlayouts * 1000 / thinkingProgress.totalTimeMs
    return (
      <div>
        <progress value={thinkingProgress.completed} max={thinkingProgress.required} />
        <p>{thinkingProgress.totalPlayouts.toLocaleString()} playouts</p>
        <p className="playout-counter">{`${playoutsPerSec.toFixed(0)} playouts/sec`}</p>
      </div>
    )
  }
)

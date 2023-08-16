import React from 'react'
import { type WorkerRequest, type WorkerResponse } from './WorkerProtocol'

import Board from './Board'
import type GameState from './GameState'
import type { MoveScores } from './MoveScores'
import { BOT_PLAYER, HUMAN_PLAYER, NPLAYERS } from './constants'

interface ThinkingProgress {
  completed: number
  required: number
  totalTimeMs: number
}

function useWorker (): [Worker | undefined, (r: WorkerRequest) => void] {
  const [worker, setWorker] = React.useState<Worker | undefined>(undefined)

  React.useEffect(() => {
    const initializingWorker = new Worker(
      new URL('./bot.worker.ts', import.meta.url),
      { name: 'Rules engine and AI', type: 'module' }
    )
    setWorker(initializingWorker)
    return () => {
      console.log('terminating the worker')
      initializingWorker.terminate()
      setWorker(undefined)
    }
  }, [])

  const postMessage = React.useCallback((request: WorkerRequest) => {
    if (worker != null) {
      console.log('Actually sending message: ', request)
      worker.postMessage(request)
    } else {
      console.log('Not sending message: ', request)
    }
  }, [worker])

  return [worker, postMessage]
}

function getMemoryUsageFormatter (): Intl.NumberFormat {
  return new Intl.NumberFormat(
    navigator.language,
    { notation: 'compact', style: 'unit', unit: 'byte', unitDisplay: 'narrow' }
  )
}

export default function App (): React.JSX.Element {
  const [gameState, setGameState] = React.useState<GameState | undefined>(undefined)
  const [possibleMoves, setPossibleMoves] = React.useState<number[] | undefined>(undefined)
  const [chosenCell, setChosenCell] = React.useState<number | undefined>(undefined)
  const [lastMoveInvalid, setLastMoveInvalid] = React.useState<boolean | undefined>(undefined)
  const [moveScores, setMoveScores] = React.useState<MoveScores | undefined>(undefined)
  const [thinkingProgress, setThinkingProgress] = React.useState<ThinkingProgress | undefined>(undefined)
  const [treeSize, setTreeSize] = React.useState<number>(0)
  const [memoryUsage, setMemoryUsage] = React.useState<number>(0)

  const [worker, postMessage] = useWorker()

  React.useEffect(() => {
    if (worker == null) {
      return
    }
    worker.onmessage = (event: MessageEvent) => {
      const response = event.data as WorkerResponse
      switch (response.type) {
        case 'initialized':
          console.log('Webworker finished initialization')
          setGameState(response.gameState)
          setPossibleMoves(response.possibleMoves)
          break
        case 'possibleMoves':
          setPossibleMoves(response.possibleMoves)
          break
        case 'state':
          setGameState(response.gameState)
          setThinkingProgress(undefined)
          setChosenCell(undefined)
          postMessage({ type: 'possibleMoves' })
          if (response.gameState.activePlayer === BOT_PLAYER) {
            postMessage({ type: 'takeAction' })
          }
          break
        case 'illegalMove':
        case 'illegalPlacement':
          setLastMoveInvalid(true)
          break
        case 'moveScores':
          setMoveScores({
            player: response.activePlayer,
            tally: response.moveScores
          })
          setTreeSize(response.treeSize)
          setMemoryUsage(response.memoryUsage)
          break
        case 'placeScores':
          setMoveScores({
            player: response.activePlayer,
            tally: response.placeScores
          })
          setTreeSize(response.treeSize)
          setMemoryUsage(response.memoryUsage)
          break
        case 'thinkingProgress':
          setThinkingProgress({
            completed: response.completed,
            required: response.required,
            totalTimeMs: response.totalTimeMs
          })
          break
      }
    }
  }, [worker, postMessage])

  const modeType = gameState?.modeType
  const handleCellClick = React.useCallback((key: number) => {
    if (chosenCell === key) {
      setChosenCell(undefined)
      postMessage({ type: 'possibleMoves' })
    } else if (possibleMoves?.includes(key) === true) {
      if (modeType === 'drafting') {
        postMessage({
          type: 'place',
          dst: key
        })
      } else if (modeType === 'playing') {
        if (chosenCell === undefined) {
          setChosenCell(key)
          postMessage({ type: 'possibleMoves', src: key })
        } else {
          postMessage({
            type: 'move',
            src: chosenCell,
            dst: key
          })
        }
      }
    }
  }, [postMessage, modeType, possibleMoves, chosenCell])

  const invalidMoveBlock = lastMoveInvalid === true
    ? <p>"Invalid move!"</p>
    : undefined

  const scoresBlock = []
  for (let p = 0; p < NPLAYERS; ++p) {
    const playerClass = p === HUMAN_PLAYER ? 'human' : 'bot'
    const active = gameState?.activePlayer === p ? '(Active Player)' : undefined
    scoresBlock.push(
      <p key={'score_' + p.toString()}><span className={playerClass}>
        Score: {gameState?.scores[p]} {active}
      </span></p>
    )
  }

  let topMove
  if (moveScores?.player === BOT_PLAYER && moveScores !== undefined) {
    for (const score of moveScores.tally) {
      if (topMove === undefined || score.rewards > topMove.rewards) {
        topMove = score
      }
    }
  }

  let board
  if (gameState != null) {
    board = <Board
      gameState={gameState}
      possibleMoves={possibleMoves ?? []}
      chosenCell={chosenCell}
      handleCellClick={handleCellClick}
      topMove={topMove}
    />
  }

  let winChanceMeter
  if (moveScores !== undefined) {
    let totalVisits = 0
    let totalRewards = 0
    for (const mov of moveScores.tally) {
      totalVisits += mov.visits
      totalRewards += mov.rewards
    }

    let chance = totalRewards / totalVisits
    if (moveScores.player !== HUMAN_PLAYER) {
      chance = 1 - chance
    }

    winChanceMeter = (
      <div className="win-chance">
        <meter id="win-chance-bar" min={0} max={1} low={0.49} high={0.5} optimum={1} value={chance} />
        <label htmlFor="win-chance-bar">Who's winning? (higher is better for you)</label>
      </div>
    )
  }

  let thinkingProgressBar
  if (thinkingProgress !== undefined) {
    const playoutsPerSec = thinkingProgress.completed * 1000 / thinkingProgress.totalTimeMs
    thinkingProgressBar = (
      <div>
        <progress value={thinkingProgress.completed} max={thinkingProgress.required} />
        <p className="playout-counter">{`${playoutsPerSec.toFixed(0)} playouts/sec`}</p>
      </div>
    )
  }

  const memoryUsageFormatter = React.useMemo(
    getMemoryUsageFormatter,
    [navigator.language]
  )

  return (
    <div className="app">
      {board}
      <div className="info-col">
        <p>{gameState?.modeType}</p>
        {invalidMoveBlock}
        <div>{scoresBlock}</div>
        {winChanceMeter}
        <p className="tree-size">Stored {treeSize.toLocaleString()} game states in memory</p>
        <p className="memory-usage">Using {memoryUsageFormatter.format(memoryUsage)}</p>
        <p className="memory-ratio">Ratio is {memoryUsageFormatter.format(memoryUsage / treeSize)} per state</p>
        {thinkingProgressBar}
      </div>
    </div>
  )
}

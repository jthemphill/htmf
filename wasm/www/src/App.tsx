import React from "react";

import Board from "./Board";
import { BOT_PLAYER, HUMAN_PLAYER, NPLAYERS } from "./constants";
import {
  type GameState,
  type PlayerMoveScores,
  type BotWorker,
  type WorkerResponse,
} from "./WorkerProtocol";

interface ThinkingProgress {
  completed: number;
  required: number;
  totalPlayouts: number;
  totalTimeMs: number;
}

type WorkerState = {
  gameState?: GameState;
  possibleMoves?: number[];
  lastMoveWasIllegal?: boolean;
  playerMoveScores?: PlayerMoveScores;
  thinkingProgress?: ThinkingProgress;
  treeSize?: number;
  memoryUsage?: number;
};

const WorkerStateReducer = (
  state: WorkerState,
  response: WorkerResponse,
): WorkerState => {
  switch (response.type) {
    case "gameState":
      return {
        ...state,
        gameState: response.gameState,
        possibleMoves: response.possibleMoves,
        lastMoveWasIllegal: response.lastMoveWasIllegal,
        playerMoveScores: undefined,
      };
    case "thinkingProgress":
      return {
        ...state,
        playerMoveScores: response.playerMoveScores,
        treeSize: response.treeSize,
        memoryUsage: response.memoryUsage,
        thinkingProgress: {
          completed: response.completed,
          required: response.required,
          totalPlayouts: response.totalPlayouts,
          totalTimeMs: response.totalTimeThinkingMs,
        },
      };
  }
};

const useWorker = (): [React.RefObject<BotWorker>, WorkerState] => {
  const [state, dispatch] = React.useReducer(WorkerStateReducer, {});
  const workerRef = React.useRef<BotWorker | null>(null);

  React.useEffect(() => {
    if (!workerRef.current) {
      const worker = new Worker(new URL("./bot.worker.ts", import.meta.url), {
        name: "Rules engine and AI",
        type: "module",
      }) as BotWorker;
      worker.onmessage = ({ data: response }) => {
        dispatch(response);
      };
      worker.postMessage({ type: "getGameState" });
      workerRef.current = worker;
    }

    return () => {
      workerRef.current?.terminate();
      workerRef.current = null;
    };
  }, []);

  return [workerRef, state];
};

export default function App({}): React.JSX.Element {
  const [
    workerRef,
    {
      gameState,
      possibleMoves,
      lastMoveWasIllegal,
      playerMoveScores,
      thinkingProgress,
      treeSize,
      memoryUsage,
    },
  ] = useWorker();
  const [chosenCell, setChosenCell] = React.useState<number | undefined>(
    undefined,
  );

  const modeType = gameState?.modeType;
  const handleCellClick = React.useCallback(
    function handleCellClick(key: number) {
      const worker: BotWorker | null = workerRef.current;
      if (worker === null) {
        throw new TypeError("Couldn't initialize the WebWorker");
      }

      // Reset state if we know it's not a legal move
      if (modeType === undefined || possibleMoves?.includes(key) !== true) {
        setChosenCell(undefined);
        workerRef.current?.postMessage({ type: "getPossibleMoves" });
        return;
      }

      // Place a penguin if we're still drafting
      if (modeType === "drafting") {
        worker.postMessage({
          type: "movePenguin",
          dst: key,
        });
        return;
      }

      // Choose the given cell to move a penguin from
      if (chosenCell === undefined) {
        setChosenCell(key);
        worker.postMessage({ type: "getPossibleMoves", src: key });
        return;
      }

      // If we have already chosen a penguin, move the penguin to the clicked cell
      worker.postMessage({
        type: "movePenguin",
        src: chosenCell,
        dst: key,
      });
      setChosenCell(undefined);
    },
    [workerRef, modeType, possibleMoves, chosenCell],
  );

  let topMove;
  if (
    playerMoveScores !== undefined &&
    playerMoveScores.player === BOT_PLAYER
  ) {
    for (const score of playerMoveScores.moveScores) {
      if (topMove === undefined || score.rewards > topMove.rewards) {
        topMove = score;
      }
    }
  }

  return (
    <div className="app">
      {gameState !== undefined && (
        <Board
          gameState={gameState}
          possibleMoves={possibleMoves ?? []}
          chosenCell={chosenCell}
          handleCellClick={handleCellClick}
          topMove={topMove}
        />
      )}
      <div className="info-col">
        <p>{gameState?.modeType}</p>
        {lastMoveWasIllegal && <p>"Invalid move!"</p>}
        {gameState !== undefined && (
          <ScoresBlock
            activePlayer={gameState.activePlayer}
            scores={gameState.scores}
          />
        )}
        {playerMoveScores !== undefined && (
          <WinChanceMeter playerMoveScores={playerMoveScores} />
        )}
        <MemoryUsageBlock
          memoryUsage={memoryUsage ?? 0}
          treeSize={treeSize ?? 0}
        />
        {thinkingProgress !== undefined && (
          <ThinkingProgressBar thinkingProgress={thinkingProgress} />
        )}
      </div>
    </div>
  );
}

const ScoresBlock = React.memo(function ScoresBlock({
  activePlayer,
  scores,
}: {
  activePlayer?: number;
  scores: number[];
}): React.JSX.Element {
  const scoresBlock = [];
  for (let p = 0; p < NPLAYERS; ++p) {
    const playerClass = p === HUMAN_PLAYER ? "human" : "bot";
    const active = activePlayer === p ? "(Active Player)" : undefined;
    scoresBlock.push(
      <p key={"score_" + p.toString()}>
        <span className={playerClass}>
          Score: {scores[p]} {active}
        </span>
      </p>,
    );
  }
  return <div className="scores-block">{scoresBlock}</div>;
});

const WinChanceMeter = React.memo(function WinChanceMeter({
  playerMoveScores,
}: {
  playerMoveScores: PlayerMoveScores;
}): React.JSX.Element {
  let totalVisits = 0;
  let totalRewards = 0;
  for (const mov of playerMoveScores.moveScores) {
    totalVisits += mov.visits;
    totalRewards += mov.rewards;
  }

  let chance = totalVisits > 0 ? totalRewards / totalVisits : 0.5;
  if (playerMoveScores.player !== HUMAN_PLAYER) {
    chance = 1 - chance;
  }

  return (
    <div className="win-chance">
      <meter
        id="win-chance-bar"
        min={0}
        max={1}
        low={0.49}
        high={0.5}
        optimum={1}
        value={chance}
      />
      <label htmlFor="win-chance-bar">
        Who's winning? (higher is better for you)
      </label>
    </div>
  );
});

const MemoryUsageBlock = React.memo(function MemoryUsageBlock({
  memoryUsage,
  treeSize,
}: {
  memoryUsage: number;
  treeSize: number;
}): React.JSX.Element {
  const memoryUsageFormatter = new Intl.NumberFormat(navigator.language, {
    notation: "compact",
    style: "unit",
    unit: "byte",
    unitDisplay: "narrow",
  });
  return (
    <>
      <p className="tree-size">
        Stored {treeSize.toLocaleString()} game states in memory
      </p>
      <div className="memory-usage-block">
        <p className="memory-usage">
          Using {memoryUsageFormatter.format(memoryUsage)}
        </p>
        <p className="memory-ratio">
          Ratio is {memoryUsageFormatter.format(memoryUsage / treeSize)} per
          state
        </p>
      </div>
    </>
  );
});

const ThinkingProgressBar = React.memo(function ThinkingProgressBar({
  thinkingProgress,
}: {
  thinkingProgress: ThinkingProgress;
}): React.JSX.Element {
  const totalTimeSec = thinkingProgress.totalTimeMs / 1000;
  const playoutsPerSec = thinkingProgress.totalPlayouts / totalTimeSec;
  return (
    <div>
      <progress
        value={thinkingProgress.completed}
        max={thinkingProgress.required}
      />
      <p>
        {thinkingProgress.totalPlayouts.toLocaleString()} playouts in{" "}
        {totalTimeSec.toFixed(0)} sec
      </p>
      <p className="playout-counter">{`${playoutsPerSec.toFixed(0)} playouts/sec`}</p>
    </div>
  );
});

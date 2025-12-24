import * as React from "react";
import { BOT_PLAYER, HUMAN_PLAYER, NPLAYERS } from "../browser/constants";
import Board from "./Board";
import "./index.css";
function WorkerStateReducer(state, response) {
    switch (response.type) {
        case "initialized":
            return {
                ...state,
                worker: response.worker,
            };
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
        case "terminated":
            return {
                ...state,
                worker: undefined,
            };
    }
}
function useWorker() {
    const [workerState, workerDispatch] = React.useReducer(WorkerStateReducer, {});
    React.useEffect(function workerLifecycle() {
        const worker = new Worker(new URL("../webworker/bot.worker.ts", import.meta.url), {
            name: "Rules engine and AI",
            type: "module",
        });
        worker.onerror = (ev) => {
            throw new Error("WebWorker failure", { cause: ev });
        };
        worker.onmessage = ({ data: response }) => {
            workerDispatch(response);
        };
        worker.postMessage({ type: "getGameState" });
        workerDispatch({
            type: "initialized",
            worker,
        });
        return () => {
            workerDispatch({
                type: "terminated",
            });
            worker.terminate();
        };
    }, []);
    return workerState;
}
export default function App() {
    const { worker, gameState, possibleMoves, lastMoveWasIllegal, playerMoveScores, thinkingProgress, treeSize, memoryUsage, } = useWorker();
    const [chosenCell, setChosenCell] = React.useState(undefined);
    const modeType = gameState?.modeType;
    function handleCellClick(key) {
        if (worker === undefined) {
            throw new TypeError("Couldn't initialize the WebWorker");
        }
        // Reset state if we know it's not a legal move
        if (modeType === undefined || possibleMoves?.includes(key) !== true) {
            setChosenCell(undefined);
            worker.postMessage({ type: "getPossibleMoves" });
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
    }
    let topMove;
    if (playerMoveScores?.player === BOT_PLAYER) {
        for (const score of playerMoveScores.moveScores) {
            if (topMove === undefined || score.rewards > topMove.rewards) {
                topMove = score;
            }
        }
    }
    return (React.createElement("div", { className: "app" },
        gameState !== undefined && (React.createElement(Board, { gameState: gameState, possibleMoves: possibleMoves ?? [], chosenCell: chosenCell, handleCellClick: handleCellClick, topMove: topMove })),
        React.createElement("div", { className: "info-col" },
            React.createElement("p", null, gameState?.modeType),
            lastMoveWasIllegal && React.createElement("p", null, "\"Invalid move!\""),
            gameState !== undefined && (React.createElement(ScoresBlock, { activePlayer: gameState.activePlayer, scores: gameState.scores })),
            playerMoveScores !== undefined && (React.createElement(WinChanceMeter, { playerMoveScores: playerMoveScores })),
            React.createElement(MemoryUsageBlock, { memoryUsage: memoryUsage ?? 0, treeSize: treeSize ?? 0 }),
            thinkingProgress !== undefined && (React.createElement(ThinkingProgressBar, { thinkingProgress: thinkingProgress })))));
}
function ScoresBlock({ activePlayer, scores, }) {
    const scoresBlock = [];
    for (let p = 0; p < NPLAYERS; ++p) {
        const playerClass = p === HUMAN_PLAYER ? "human" : "bot";
        const active = activePlayer === p ? "(Active Player)" : undefined;
        scoresBlock.push(React.createElement("p", { key: "score_" + p.toString() },
            React.createElement("span", { className: playerClass },
                "Score: ",
                scores[p],
                " ",
                active)));
    }
    return React.createElement("div", { className: "scores-block" }, scoresBlock);
}
function WinChanceMeter({ playerMoveScores, }) {
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
    return (React.createElement("div", { className: "win-chance" },
        React.createElement("meter", { id: "win-chance-bar", min: 0, max: 1, low: 0.49, high: 0.5, optimum: 1, value: chance }),
        React.createElement("label", { htmlFor: "win-chance-bar" }, "Who's winning? (higher is better for you)")));
}
function MemoryUsageBlock({ memoryUsage, treeSize, }) {
    const memoryUsageFormatter = new Intl.NumberFormat(undefined, {
        notation: "compact",
        style: "unit",
        unit: "byte",
        unitDisplay: "narrow",
    });
    return (React.createElement(React.Fragment, null,
        React.createElement("p", { className: "tree-size" },
            "Stored ",
            treeSize.toLocaleString(),
            " game states in memory"),
        React.createElement("div", { className: "memory-usage-block" },
            React.createElement("p", { className: "memory-usage" },
                "Using ",
                memoryUsageFormatter.format(memoryUsage)),
            React.createElement("p", { className: "memory-ratio" },
                "Ratio is ",
                memoryUsageFormatter.format(memoryUsage / treeSize),
                " per state"))));
}
function ThinkingProgressBar({ thinkingProgress, }) {
    const totalTimeSec = thinkingProgress.totalTimeMs / 1000;
    const playoutsPerSec = thinkingProgress.totalPlayouts / totalTimeSec;
    return (React.createElement("div", null,
        React.createElement("progress", { value: thinkingProgress.completed, max: thinkingProgress.required }),
        React.createElement("p", null,
            thinkingProgress.totalPlayouts.toLocaleString(),
            " playouts in",
            " ",
            totalTimeSec.toFixed(0),
            " sec"),
        React.createElement("p", { className: "playout-counter" }, `${playoutsPerSec.toFixed(0)} playouts/sec`)));
}

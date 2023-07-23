import React from "react";
import { createRoot } from "react-dom/client";
import { WorkerRequest, WorkerResponse } from "./WorkerProtocol";

import Board from "./Board";
import GameState from "./GameState";
import { BOT_PLAYER, HUMAN_PLAYER, NPLAYERS } from "./constants";

type MoveScores = {
    player: number,
    tally: { src?: number, dst: number, visits: number, rewards: number }[]
};

type ThinkingProgress = {
    completed: number,
    required: number,
};

function useWorker(): [Worker | undefined, (r: WorkerRequest) => void] {
    const [worker, setWorker] = React.useState<Worker | undefined>(undefined);

    React.useEffect(() => {
        const initializingWorker = new Worker(
            new URL("./bot.worker.js", import.meta.url),
            { name: "Rules engine and AI" },
        );
        setWorker(initializingWorker);
        return () => {
            console.log('terminating the worker');
            initializingWorker.terminate();
            setWorker(undefined);
        };
    }, []);

    const postMessage = React.useCallback((request: WorkerRequest) => {
        if (worker) {
            console.log("Actually sending message: ", request);
            worker.postMessage(request);
        } else {
            console.log("Not sending message: ", request);
        }
    }, [worker]);

    return [worker, postMessage];
}

const App = function () {

    const [gameState, setGameState] = React.useState<GameState | undefined>(undefined);
    const [possibleMoves, setPossibleMoves] = React.useState<number[] | undefined>(undefined);
    const [chosenCell, setChosenCell] = React.useState<number | undefined>(undefined);
    const [lastMoveInvalid, setLastMoveInvalid] = React.useState<boolean | undefined>(undefined);
    const [moveScores, setMoveScores] = React.useState<MoveScores | undefined>(undefined);
    const [thinkingProgress, setThinkingProgress] = React.useState<ThinkingProgress | undefined>(undefined);

    const [worker, postMessage] = useWorker();

    React.useEffect(() => {
        if (!worker) {
            return;
        }
        worker.onmessage = (event: MessageEvent) => {
            const response = event.data as WorkerResponse;
            console.log(`Received message from WebWorker: ${response.type}`);
            switch (response.type) {
                case "initialized":
                    console.log("Webworker finished initialization");
                    setGameState(response.gameState);
                    setPossibleMoves(response.possibleMoves);
                    break;
                case "possibleMoves":
                    setPossibleMoves(response.possibleMoves);
                    break;
                case "state":
                    setGameState(response.gameState);
                    setThinkingProgress(undefined);
                    setChosenCell(undefined);
                    postMessage({ type: "possibleMoves" });
                    if (response.gameState.activePlayer === BOT_PLAYER) {
                        postMessage({ type: "takeAction" });
                    }
                    break;
                case "illegalMove":
                case "illegalPlacement":
                    setLastMoveInvalid(true);
                    break;
                case "moveScores":
                    setMoveScores({
                        player: response.activePlayer,
                        tally: response.moveScores
                    });
                    break;
                case "placeScores":
                    setMoveScores({
                        player: response.activePlayer,
                        tally: response.placeScores
                    });
                    break;
                case "thinkingProgress":
                    setThinkingProgress({
                        completed: response.completed,
                        required: response.required,
                    });
                    break;
            }
        };
    }, [worker, postMessage]);

    const handleCellClick = React.useCallback((key: number) => {
        if (gameState === undefined) {
            return;
        }
        if (gameState.activePlayer === undefined) {
            return;
        }
        if (gameState.modeType === "drafting") {
            if (possibleMoves?.includes(key)) {
                postMessage({
                    type: "place",
                    dst: key,
                });
            }
        } else {
            if (chosenCell !== undefined && possibleMoves?.includes(key)) {
                if (chosenCell !== undefined) {
                    postMessage({
                        type: "move",
                        src: chosenCell,
                        dst: key,
                    });
                };
            } else if (gameState.board.penguins[gameState.activePlayer]?.includes(key)) {
                if (chosenCell === key) {
                    setChosenCell(undefined);
                    postMessage({ type: "possibleMoves" });
                    return;
                }

                setChosenCell(key);
                postMessage({ type: "possibleMoves", src: key });
            }
        }
    }, [postMessage, gameState, possibleMoves, chosenCell]);

    const invalidMoveBlock = lastMoveInvalid
        ? <p>"Invalid move!"</p>
        : undefined;

    const scoresBlock = [];
    for (let p = 0; p < NPLAYERS; ++p) {
        let playerClass = p === HUMAN_PLAYER ? "human" : "bot";
        let active = gameState?.activePlayer === p ? '(Active Player)' : undefined;
        scoresBlock.push(
            <p key={"score_" + p}><span className={playerClass}>
                Score: {gameState?.scores[p]} {active}
            </span></p>
        );
    }

    let board = undefined;
    if (gameState) {
        board = <Board
            gameState={gameState}
            possibleMoves={possibleMoves || []}
            chosenCell={chosenCell}
            handleCellClick={handleCellClick}
        />;
    }

    let winChanceMeter = undefined;
    if (moveScores !== undefined) {
        let totalVisits = 0;
        let totalRewards = 0;
        for (let mov of moveScores.tally) {
            totalVisits += mov.visits;
            totalRewards += mov.rewards;
        }

        let chance = totalRewards / totalVisits;
        if (moveScores.player !== HUMAN_PLAYER) {
            chance = 1 - chance;
        }

        winChanceMeter = <meter min={0} max={1} low={0.49} high={0.5} optimum={1} value={chance} />;
    }

    let thinkingProgressBar = undefined;
    if (thinkingProgress !== undefined) {
        thinkingProgressBar = <progress value={thinkingProgress.completed} max={thinkingProgress.required} />;
    }

    return (
        <div className="app">
            {board}
            <div className="info-col">
                <p>{gameState?.modeType}</p>
                {invalidMoveBlock}
                <div>{scoresBlock}</div>
                {winChanceMeter}
                {thinkingProgressBar}
            </div>
        </div>
    );
};

const container = document.getElementById('root');
if (container) {
    const root = createRoot(container);
    root.render(
        <React.StrictMode>
            <App />
        </React.StrictMode>
    );
}
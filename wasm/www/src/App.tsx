import React from "react";
import ReactDOM from "react-dom";
import { WorkerRequest, WorkerResponse } from "./WorkerProtocol";

import Board from "./Board";
import GameState from "./GameState";
import { BOT_PLAYER, HUMAN_PLAYER, NPLAYERS } from "./constants";

type State = {
    gameState?: GameState,
    possibleMoves?: number[],
    chosenCell?: number,
    lastMoveInvalid?: boolean,
    moveScores?: {
        player: number,
        tally: { src?: number, dst: number, visits: number, rewards: number }[]
    },
    thinkingProgress?: {
        completed: number,
        required: number,
    },
};
type Props = {

};
class App extends React.Component<Props, State> {

    handleCellClick: ((idx: number) => void);
    state: State;
    worker: Worker;

    constructor(props: Props) {
        super(props);
        this.handleCellClick = this._handleCellClick.bind(this);
        this.worker = new Worker(new URL("./bot.worker.ts", import.meta.url));
        this.worker.onmessage = this.onMessage.bind(this);
        this.state = {};
    }

    activePlayer() {
        return this.state.gameState?.activePlayer;
    }

    render() {
        const invalid_move_block = this.state.lastMoveInvalid
            ? "Invalid move!"
            : undefined;

        const scores_block = [];
        const active_player = this.activePlayer();
        for (let p = 0; p < NPLAYERS; ++p) {
            let playerClass = p === HUMAN_PLAYER ? "human" : "bot";
            let active = active_player === p ? '(Active Player)' : undefined;
            scores_block.push(
                <p key={"score_" + p}><span className={playerClass}>
                    Score: {this.state.gameState?.scores[p]} {active}
                </span></p>
            );
        }

        let board = undefined;
        if (this.state.gameState) {
            board = <Board
                gameState={this.state.gameState}
                possibleMoves={this.state.possibleMoves || []}
                chosenCell={this.state.chosenCell}
                handleCellClick={this.handleCellClick}
            />;
        }

        const moveScores = this.state.moveScores;
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

        const thinkingProgress = this.state.thinkingProgress;
        let thinkingProgressBar = undefined;
        if (thinkingProgress !== undefined) {
            thinkingProgressBar = <progress value={thinkingProgress.completed} max={thinkingProgress.required} />;
        }

        return (
            <div className="app">
                {board}
                <div className="info-col">
                    <p>{this.state.gameState?.modeType}</p>
                    <p>{invalid_move_block}</p>
                    <div>{scores_block}</div>
                    {winChanceMeter}
                    {thinkingProgressBar}
                </div>
            </div>
        );
    }

    componentDidMount() {
        this.postMessage({ type: "get" });
        this.postMessage({ type: "possibleMoves" });
        this.postMessage({ type: "startPondering" });
    }

    componentWillUnmount() {
        this.worker.terminate();
    }

    _handleCellClick(key: number) {
        const activePlayer = this.activePlayer();
        if (activePlayer === undefined) {
            return;
        }
        const gameState = this.state.gameState;
        if (gameState === undefined) {
            return;
        }
        if (gameState.modeType === "drafting") {
            if (this.state.possibleMoves?.includes(key)) {
                this._placePenguin(key);
            }
        } else {
            if (this.state.chosenCell !== undefined && this.state.possibleMoves?.includes(key)) {
                this._movePenguinToCell(key);
            } else if (gameState.board.penguins[activePlayer]?.includes(key)) {
                this._toggleCellHighlight(key);
            }
        }
    }

    postMessage(request: WorkerRequest) {
        console.log(`sent request ${request.type}`);
        this.worker.postMessage(request);
    }

    onMessage(event: MessageEvent) {
        const response = event.data as WorkerResponse;
        switch (response.type) {
            case "possibleMoves":
                this.setState({ possibleMoves: response.possibleMoves });
                break;
            case "state":
                this.setState({
                    gameState: response.gameState,
                    thinkingProgress: undefined,
                    chosenCell: undefined,
                });
                this.postMessage({ type: "possibleMoves" });
                if (response.gameState.activePlayer === BOT_PLAYER) {
                    this.postMessage({ type: "takeAction" });
                }
                break;
            case "illegalMove":
            case "illegalPlacement":
                this.setState({ lastMoveInvalid: true });
                break;
            case "moveScores":
                this.setState({
                    moveScores: {
                        player: response.activePlayer,
                        tally: response.moveScores
                    }
                });
                break;
            case "placeScores":
                this.setState({
                    moveScores: {
                        player: response.activePlayer,
                        tally: response.placeScores
                    }
                });
                break;
            case "thinkingProgress":
                this.setState({
                    thinkingProgress: {
                        completed: response.completed,
                        required: response.required,
                    },
                });
                break;
        }
    }

    _placePenguin(key: number) {
        this.postMessage({
            type: "place",
            dst: key,
        });
    }

    _movePenguinToCell(key: number) {
        if (this.state.chosenCell !== undefined) {
            this.postMessage({
                type: "move",
                src: this.state.chosenCell,
                dst: key,
            });
        }
    }

    _toggleCellHighlight(key: number) {
        if (this.state.chosenCell === key) {
            this.setState({
                chosenCell: undefined,
            });
            this.postMessage({ type: "possibleMoves" });
            return;
        }

        this.postMessage({ type: "possibleMoves", src: key });
        this.setState({
            chosenCell: key,
        });
    }
}

ReactDOM.render(<App />, document.getElementById('root'));
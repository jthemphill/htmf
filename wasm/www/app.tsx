import React from "react";
import ReactDOM from "react-dom";
import Worker from "worker-loader!./bot.worker";
import { WorkerRequest, WorkerResponse } from "./WorkerProtocol";

import Board from "./Board";
import GameState from "./GameState";
import { BOT_PLAYER, NPLAYERS, NUM_CELLS, PLAYER_COLORS, PONDER_INTERVAL_MS, PLAYOUT_MS } from "./constants";

type State = {
    gameState?: GameState,
    possibleMoves?: number[],
    chosenCell?: number,
    lastMoveInvalid?: boolean,
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
        this.worker = new Worker();
        this.worker.onmessage = this.onMessage.bind(this);
        this.state = {};
    }

    activePlayer() {
        return this.state.gameState?.activePlayer;
    }

    render() {
        const invalid_move_block = this.state.lastMoveInvalid
            ? "Invalid move!"
            : null;

        const scores_block = [];
        const active_player = this.activePlayer();
        for (let p = 0; p < NPLAYERS; ++p) {
            let color = { color: PLAYER_COLORS[p] };
            let active = active_player === p ? '(Active Player)' : null;
            scores_block.push(
                <p key={"score_" + p}><span style={color}>
                    Score: {this.state.gameState?.scores[p]} {active}
                </span></p>
            );
        }

        let board = null;
        if (this.state.gameState) {
            board = <Board
                gameState={this.state.gameState}
                possibleMoves={this.state.possibleMoves || []}
                chosenCell={this.state.chosenCell}
                handleCellClick={this.handleCellClick}
            />;
        }

        return (
            <div className="app">
                {board}
                <div className="info-col">
                    <p>{this.state.gameState?.modeType}</p>
                    <p>{invalid_move_block}</p>
                    <div>{scores_block}</div>
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
        if (activePlayer == null) {
            return;
        }
        if (this.state.gameState?.modeType === "drafting") {
            this._placePenguin(key);
            return;
        }
        if (this.state.gameState?.board.penguins[activePlayer].includes(key)) {
            this._toggleCellHighlight(key);
            return;
        }
        if (this.state.chosenCell == null) {
            return;
        }
        if (this.state.possibleMoves?.includes(key)) {
            this._movePenguinToCell(key);
            return;
        }
    }

    postMessage(request: WorkerRequest) {
        console.log(`sent request ${request}`);
        this.worker.postMessage(request);
    }

    onMessage(event: MessageEvent) {
        const response = event.data as WorkerResponse;
        switch (response.type) {
            case "possibleMoves":
                this.setState({ possibleMoves: response.possibleMoves });
                break;
            case "state":
                this.setState({ gameState: response.gameState });
                this.postMessage({ type: "possibleMoves" });
                if (response.gameState.activePlayer === BOT_PLAYER) {
                    this.postMessage({ type: "takeAction" });
                }
                break;
            case "illegalMove":
            case "illegalPlacement":
                this.setState({ lastMoveInvalid: true });
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
        if (this.state.chosenCell) {
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
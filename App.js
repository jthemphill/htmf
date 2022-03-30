import React from "react";
import ReactDOM from "react-dom";
import Board from "./Board";
import { BOT_PLAYER, HUMAN_PLAYER, NPLAYERS } from "./constants";
class App extends React.Component {
    constructor(props) {
        super(props);
        Object.defineProperty(this, "handleCellClick", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: void 0
        });
        Object.defineProperty(this, "state", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: void 0
        });
        Object.defineProperty(this, "worker", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: void 0
        });
        this.handleCellClick = this._handleCellClick.bind(this);
        this.worker = new Worker(new URL("./bot.worker.js", import.meta.url), { name: "Rules engine and AI" });
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
            scores_block.push(React.createElement("p", { key: "score_" + p },
                React.createElement("span", { className: playerClass },
                    "Score: ",
                    this.state.gameState?.scores[p],
                    " ",
                    active)));
        }
        let board = undefined;
        if (this.state.gameState) {
            board = React.createElement(Board, { gameState: this.state.gameState, possibleMoves: this.state.possibleMoves || [], chosenCell: this.state.chosenCell, handleCellClick: this.handleCellClick });
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
            winChanceMeter = React.createElement("meter", { min: 0, max: 1, low: 0.49, high: 0.5, optimum: 1, value: chance });
        }
        const thinkingProgress = this.state.thinkingProgress;
        let thinkingProgressBar = undefined;
        if (thinkingProgress !== undefined) {
            thinkingProgressBar = React.createElement("progress", { value: thinkingProgress.completed, max: thinkingProgress.required });
        }
        return (React.createElement("div", { className: "app" },
            board,
            React.createElement("div", { className: "info-col" },
                React.createElement("p", null, this.state.gameState?.modeType),
                React.createElement("p", null, invalid_move_block),
                React.createElement("div", null, scores_block),
                winChanceMeter,
                thinkingProgressBar)));
    }
    componentDidMount() {
        this.postMessage({ type: "get" });
        this.postMessage({ type: "possibleMoves" });
        this.postMessage({ type: "startPondering" });
    }
    componentWillUnmount() {
        this.worker.terminate();
    }
    _handleCellClick(key) {
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
        }
        else {
            if (this.state.chosenCell !== undefined && this.state.possibleMoves?.includes(key)) {
                this._movePenguinToCell(key);
            }
            else if (gameState.board.penguins[activePlayer]?.includes(key)) {
                this._toggleCellHighlight(key);
            }
        }
    }
    postMessage(request) {
        console.log(`sent request ${request.type}`);
        this.worker.postMessage(request);
    }
    onMessage(event) {
        const response = event.data;
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
    _placePenguin(key) {
        this.postMessage({
            type: "place",
            dst: key,
        });
    }
    _movePenguinToCell(key) {
        if (this.state.chosenCell !== undefined) {
            this.postMessage({
                type: "move",
                src: this.state.chosenCell,
                dst: key,
            });
        }
    }
    _toggleCellHighlight(key) {
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
ReactDOM.render(React.createElement(App, null), document.getElementById('root'));
//# sourceMappingURL=App.js.map
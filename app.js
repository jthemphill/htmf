"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const react_1 = __importDefault(require("react"));
const react_dom_1 = __importDefault(require("react-dom"));
const bot_worker_1 = __importDefault(require("worker-loader!./bot.worker"));
const Board_1 = __importDefault(require("./Board"));
const constants_1 = require("./constants");
class App extends react_1.default.Component {
    constructor(props) {
        super(props);
        this.handleCellClick = this._handleCellClick.bind(this);
        this.worker = new bot_worker_1.default();
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
        for (let p = 0; p < constants_1.NPLAYERS; ++p) {
            let color = { color: constants_1.PLAYER_COLORS[p] };
            let active = active_player === p ? '(Active Player)' : null;
            scores_block.push(react_1.default.createElement("p", { key: "score_" + p },
                react_1.default.createElement("span", { style: color },
                    "Score: ",
                    this.state.gameState?.scores[p],
                    " ",
                    active)));
        }
        let board = null;
        if (this.state.gameState) {
            board = react_1.default.createElement(Board_1.default, { gameState: this.state.gameState, possibleMoves: this.state.possibleMoves || [], chosenCell: this.state.chosenCell, handleCellClick: this.handleCellClick });
        }
        const moveScores = this.state.moveScores;
        if (moveScores != null) {
            let totalVisits = 0;
            let totalRewards = 0;
            for (let mov of moveScores) {
                totalVisits += mov.visits;
                totalRewards += mov.rewards;
            }
            console.log(`${totalRewards} / ${totalVisits} ~= ${totalRewards / totalVisits}`);
        }
        return (react_1.default.createElement("div", { className: "app" },
            board,
            react_1.default.createElement("div", { className: "info-col" },
                react_1.default.createElement("p", null, this.state.gameState?.modeType),
                react_1.default.createElement("p", null, invalid_move_block),
                react_1.default.createElement("div", null, scores_block))));
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
                this.setState({ gameState: response.gameState });
                this.postMessage({ type: "possibleMoves" });
                if (response.gameState.activePlayer === constants_1.BOT_PLAYER) {
                    this.postMessage({ type: "takeAction" });
                }
                break;
            case "illegalMove":
            case "illegalPlacement":
                this.setState({ lastMoveInvalid: true });
                break;
            case "moveScores":
                this.setState({ moveScores: response.moveScores });
                break;
            case "placeScores":
                this.setState({ moveScores: response.placeScores });
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
        if (this.state.chosenCell) {
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
react_dom_1.default.render(react_1.default.createElement(App, null), document.getElementById('root'));
//# sourceMappingURL=app.js.map
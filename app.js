"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const react_1 = __importDefault(require("react"));
const react_dom_1 = __importDefault(require("react-dom"));
const Board_1 = __importDefault(require("./Board"));
require("./App.css");
function shuffle(a) {
    for (let i = a.length; i; i--) {
        const j = Math.floor(Math.random() * i);
        const tmp = a[i - 1];
        a[i - 1] = a[j];
        a[j] = tmp;
    }
}
class App extends react_1.default.Component {
    constructor(props) {
        super(props);
        this.handleCellClick = this._handleCellClick.bind(this);
        const fish = [];
        for (let i = 0; i < 30; ++i) {
            fish.push(1);
        }
        for (let i = 30; i < 50; ++i) {
            fish.push(2);
        }
        for (let i = 50; i < 60; ++i) {
            fish.push(3);
        }
        shuffle(fish);
        const gameState = {
            last_move_valid: true,
            mode_type: "drafting",
            nplayers: 2,
            board: {
                active_player: 0,
                fish,
                penguins: [[], []],
                claimed: [[], []],
                possible_moves: [],
            },
            turn: 0,
            scores: [0, 0],
        };
        this.state = {
            gameState,
            inputText: '',
            minDim: Math.min(window.innerWidth, window.innerHeight),
            chosenCell: null,
            disablePossibleMoves: false,
        };
    }
    activePlayer() {
        return this.state.gameState.active_player;
    }
    render() {
        const invalid_move_block = this.state.gameState.last_move_valid
            ? null
            : "Invalid move!";
        // duplicated in board.react.js
        const player_colors = ['blue', 'red', 'orange', 'green'];
        const scores_block = [];
        const active_player = this.activePlayer();
        for (let p = 0; p < this.state.gameState.nplayers; ++p) {
            let color = { color: player_colors[p] };
            let active = active_player === p ? '(Active Player)' : null;
            scores_block.push(react_1.default.createElement("p", { key: "score_" + p },
                react_1.default.createElement("span", { style: color },
                    "Score: ",
                    this.state.gameState.scores[p],
                    " ",
                    active)));
        }
        return (react_1.default.createElement("div", { className: "App", style: { 'display': 'grid' } },
            react_1.default.createElement(Board_1.default, { socket: this.socket, gameState: this.state.gameState, minDim: this.state.minDim, disablePossibleMoves: this.state.disablePossibleMoves, chosenCell: this.state.chosenCell, handleCellClick: this.handleCellClick }),
            react_1.default.createElement("div", { className: "info-col", style: { 'gridColumn': '12 / auto' } },
                react_1.default.createElement("p", null, this.state.gameState.mode_type),
                react_1.default.createElement("p", null, invalid_move_block),
                react_1.default.createElement("div", null, scores_block),
                react_1.default.createElement("input", { value: this.state.inputText, type: "text", onChange: (e) => {
                        this.setState({ inputText: e.target.value });
                    }, onBlur: (e) => {
                        try {
                            const input_data = JSON.parse(this.state.inputText);
                            this.socket.send(JSON.stringify({
                                "action_type": "setup",
                                "data": { state: input_data },
                            }));
                        }
                        catch (_) {
                            return;
                        }
                    } }))));
    }
    componentDidMount() {
        this.socket = new WebSocket('ws://127.0.0.1:2794');
        this.socket.onmessage = (e) => {
            if (typeof e.data !== 'string') {
                throw TypeError([
                    'TypeError:',
                    JSON.stringify(e.data),
                    'is not a string',
                ].join(' '));
            }
            const new_state = JSON.parse(e.data);
            if (new_state.board.possible_moves === null) {
                new_state.board.possible_moves = [];
            }
            console.log('Received state: ', new_state);
            this.setState({
                gameState: new_state,
                inputText: JSON.stringify(new_state),
                disablePossibleMoves: false,
            });
        };
        window.addEventListener('resize', this.updateWindowDimensions.bind(this));
    }
    componentWillUnmount() {
        this.socket.close();
        window.removeEventListener('resize', this.updateWindowDimensions.bind(this));
    }
    updateWindowDimensions() {
        this.setState({
            minDim: Math.min(window.innerWidth, window.innerHeight),
        });
    }
    _handleCellClick(key) {
        const game_state = this.state.gameState;
        const active_player = this.activePlayer();
        if (active_player === null) {
            return;
        }
        if (game_state.mode_type === 'drafting') {
            this._placePenguin(key);
        }
        else if (game_state.board.penguins[active_player].includes(key)) {
            this._toggleCellHighlight(key);
        }
        else if (this.state.chosenCell !== null &&
            game_state.board.possible_moves.includes(key)) {
            this._movePenguinToCell(key);
        }
    }
    _placePenguin(key) {
        const data = JSON.stringify({
            "action_type": "place",
            "data": {
                "hex": key,
            },
        });
        this.sendData(data);
    }
    _movePenguinToCell(key) {
        const data = JSON.stringify({
            "action_type": "move",
            "data": {
                "src": this.state.chosenCell,
                "dst": key,
            },
        });
        this.sendData(data);
    }
    _toggleCellHighlight(key) {
        if (!this.state.disablePossibleMoves && this.state.chosenCell === key) {
            this.setState({
                disablePossibleMoves: true,
            });
            return;
        }
        this.setState({
            chosenCell: key,
        });
        const data = JSON.stringify({
            "action_type": "selection",
            "data": {
                "hex": key,
            },
        });
        this.sendData(data);
    }
}
react_dom_1.default.render(react_1.default.createElement(App, null), document.getElementById('root'));
//# sourceMappingURL=app.js.map
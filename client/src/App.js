// @flow

import React from 'react';

import Board from './Board.react';
import type GameState from './GameState'

import './App.css';

function shuffle<T>(a: Array<T>): void {
    for (let i = a.length; i; i--) {
        const j = Math.floor(Math.random() * i);
        const tmp = a[i - 1];
        a[i - 1] = a[j];
        a[j] = tmp;
    }
}

type State = {
    gameState: GameState,
    inputText: string,
    minDim: number,
    chosenCell: ?number,
    // The first time we click a penguin, show moves the penguin can
    // take. The second time we click the same penguin, suppress those
    // moves.
    disablePossibleMoves: bool,
};

type Props = {

};

class App extends React.Component<Props, State> {

    handleCellClick: ((number) => void);
    socket: WebSocket;

    constructor(props: Props) {
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

    activePlayer(): number {
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
            scores_block.push(
                <p key={"score_" + p}><span style={color}>
                    Score: {this.state.gameState.scores[p]} {active}
                </span></p>
            );
        }

        return (
            <div className="App" style={{ 'display': 'grid' }}>
                <Board
                    socket={this.socket}
                    gameState={this.state.gameState}
                    minDim={this.state.minDim}
                    disablePossibleMoves={this.state.disablePossibleMoves}
                    chosenCell={this.state.chosenCell}
                    handleCellClick={this.handleCellClick}
                />
                <div className="info-col" style={{ 'gridColumn': '12 / auto' }}>
                    <p>{this.state.gameState.mode_type}</p>
                    <p>{invalid_move_block}</p>
                    <div>{scores_block}</div>
                    <input value={this.state.inputText}
                        type="text"
                        onChange={(e) => {
                            this.setState({ inputText: e.target.value });
                        }}
                        onBlur={(e) => {
                            try {
                                const input_data = JSON.parse(this.state.inputText);
                                this.socket.send(JSON.stringify({
                                    "action_type": "setup",
                                    "data": { state: input_data },
                                }));
                            } catch (_) {
                                return;
                            }
                        }} />
                </div>
            </div>
        );
    }

    componentDidMount() {
        this.socket = new WebSocket('ws://127.0.0.1:2794');
        this.socket.onmessage = (e: MessageEvent) => {
            if (typeof e.data !== 'string') {
                throw TypeError(
                    [
                        'TypeError:',
                        JSON.stringify(e.data),
                        'is not a string',
                    ].join(' ')
                );
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

    _handleCellClick(key: number) {
        const game_state = this.state.gameState;
        const active_player = this.activePlayer();
        if (active_player === null) {
            return;
        }
        if (game_state.mode_type === 'drafting') {
            this._placePenguin(key);
        } else if (game_state.board.penguins[active_player].includes(key)) {
            this._toggleCellHighlight(key);
        } else if (
            this.state.chosenCell !== null &&
            game_state.board.possible_moves.includes(key)
        ) {
            this._movePenguinToCell(key);
        }
    }

    _placePenguin(key: number) {
        const data = JSON.stringify({
            "action_type": "place",
            "data": {
                "hex": key,
            },
        });
        this.sendData(data);
    }

    _movePenguinToCell(key: number) {
        const data = JSON.stringify({
            "action_type": "move",
            "data": {
                "src": this.state.chosenCell,
                "dst": key,
            },
        });
        this.sendData(data);
    }

    _toggleCellHighlight(key: number) {
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

    sendData(data: string) {
        if (!this.socket) {
            console.log("Not connected to a server!");
            return;
        }
        console.log('sending data: ' + data);
        this.socket.send(data);
    }
}

export default App;

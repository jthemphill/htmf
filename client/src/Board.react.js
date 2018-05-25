// @flow

import * as React from 'react';

import Hex from './Hex.react';

import type GameState from './GameState';

const NUM_ROWS = 8;
const EVEN_ROW_LEN = 7;
const ODD_ROW_LEN = 8;

type Props = {
    socket: WebSocket,
    gameState: GameState,
    possibleMoves: Array<bool>,
};

type State = {
    minDim: number,
    chosenCell: ?number,
    // The first time we click a penguin, show moves the penguin can
    // take. The second time we click the same penguin, suppress those
    // moves.
    disablePossibleMoves: bool,
}

class Board extends React.Component<Props, State> {

    handleCellClick: (key: number) => void;

    constructor(props: Props) {
        super(props);

        this.state = {
            minDim: Math.min(window.innerWidth, window.innerHeight),
            chosenCell: null,
            disablePossibleMoves: false,
        };

        this.handleCellClick = this._handleCellClick.bind(this);
    }

    componentDidMount() {
        window.addEventListener('resize', this.updateWindowDimensions.bind(this));
    }

    componentWillUnmount() {
        window.removeEventListener('resize', this.updateWindowDimensions.bind(this));
    }

    updateWindowDimensions() {
        this.setState({
            minDim: Math.min(window.innerWidth, window.innerHeight),
        });
    }

    render() {
        const side_length = this.state.minDim / 16;

        const start_x = 2 * side_length;
        const start_y = 2 * side_length;

        // duplicated in app.react.js
        const player_colors = ['blue', 'red', 'orange', 'green'];
        const colors = [];
        for (
            let p = 0;
            p < this.props.gameState.board.penguins.length;
            ++p
        ) {
            let player_penguins = this.props.gameState.board.penguins[p];
            for (let c of player_penguins) {
                colors[c] = player_colors[p];
            }
        }

        let any_possible_moves = false;
        for (let can_move of this.props.possibleMoves) {
            if (can_move) {
                any_possible_moves = true;
                break;
            }
        }

        const claimed = new Set([]);
        console.log(this.props.gameState.board.claimed);
        for (let player_claimed of this.props.gameState.board.claimed) {
            for (let cell of player_claimed) {
                claimed.add(cell);
            }
        }

        const hexes = [];
        for (let r = 0; r < NUM_ROWS; ++r) {
            const y = start_y + r * 1.5 * side_length;

            const row_len = (r % 2 === 0) ? EVEN_ROW_LEN : ODD_ROW_LEN;
            const x_bobble = -1 * (r % 2) * Math.sqrt(3) * side_length / 2;

            for (let c = 0; c < row_len; ++c) {
                const hex_width = Hex.width(side_length);
                const x = start_x + c * hex_width + x_bobble;

                const key = hexes.length;
                const fish = this.props.gameState.board.fish[key];
                const color = colors[key];
                const possible = !this.state.disablePossibleMoves &&
                      any_possible_moves && this.props.possibleMoves[key];

                const is_highlighted = !this.state.disablePossibleMoves &&
                      any_possible_moves &&
                      this.state.chosenCell === key;

                hexes.push(
                  <Hex
                    key={key}
                    _key={key}
                    onClick={this.handleCellClick}
                    fish={fish}
                    cx={x}
                    cy={y}
                    sideLength={side_length}
                    highlighted={is_highlighted}
                    possible={possible}
                    color={color}
                    claimed={claimed.has(key)}
                  />
                );
            }
        }

        const style = {
            'height': this.state.minDim,
            'width': this.state.minDim,
            'gridColumn': '1 / auto',
        };

        return (
          <svg version="1.1"
            style={style}
            baseProfile="full"
            xmlns="http://www.w3.org/2000/svg">
            {hexes}
          </svg>
        );
    }

    // duplicated in App.js
    activePlayer(): ?number {
        const game_state = this.props.gameState;
        return game_state.active_player;
    }

    _handleCellClick(key: number) {
        const game_state = this.props.gameState;
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
                game_state.board.possible_moves[key]
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
            // TODO: Doing this here causes UI flicker - we
            // temporarily show the old possible moves before getting
            // the new ones from the server.
            disablePossibleMoves: false,
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
        if (!this.props.socket) {
            console.log("Not connected to a server!");
            return;
        }
        console.log('sending data: ' + data);
        this.props.socket.send(data);
    }
}

export default Board;

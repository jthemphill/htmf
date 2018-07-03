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
    minDim: number,
    disablePossibleMoves: bool,
    chosenCell: ?number,
    handleCellClick: ((number) => void),
};

type State = {

}

class Board extends React.Component<Props, State> {

    handleCellClick: (key: number) => void;

    render() {
        const side_length = this.props.minDim / 16;

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

        const any_possible_moves = this.props.gameState.board.possible_moves.length > 0;

        const claimed = new Set([]);
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
                const possible = !this.props.disablePossibleMoves &&
                    any_possible_moves && this.props.gameState.board.possible_moves.includes(key);

                const is_highlighted = !this.props.disablePossibleMoves &&
                    any_possible_moves &&
                    this.props.chosenCell === key;

                hexes.push(
                    <Hex
                        key={key}
                        _key={key}
                        onClick={this.props.handleCellClick}
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
            'height': this.props.minDim,
            'width': this.props.minDim,
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
}

export default Board;

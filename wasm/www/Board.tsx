import * as React from "react";

import Hex from "./Hex";

import GameState from "./GameState";

const NUM_ROWS = 8;
const EVEN_ROW_LEN = 7;
const ODD_ROW_LEN = 8;

type Props = {
    gameState: GameState,
    possibleMoves: number[],
    chosenCell?: number,
    handleCellClick: ((idx: number) => void),
};

export default React.memo(function (props: Props) {
    const size = 1000;
    const side_length = size / 16;

    const start_x = 2 * side_length;
    const start_y = 2 * side_length;

    // duplicated in app.react.js
    const player_colors = ['blue', 'red', 'orange', 'green'];
    const colors = [];
    for (
        let p = 0;
        p < props.gameState.board.penguins.length;
        ++p
    ) {
        let player_penguins = props.gameState.board.penguins[p];
        for (let c of player_penguins) {
            colors[c] = player_colors[p];
        }
    }

    const any_possible_moves = props.possibleMoves.length > 0;

    const claimed = new Set([]);
    for (let player_claimed of props.gameState.board.claimed) {
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
            const hex_width = side_length * (Math.sin(Math.PI / 3) - Math.sin(Math.PI * 5 / 3));
            const x = start_x + c * hex_width + x_bobble;

            const key = hexes.length;
            const fish = props.gameState.board.fish[key];
            const color = colors[key];
            const possible = props.possibleMoves.includes(key);

            const is_highlighted = any_possible_moves &&
                props.chosenCell === key;

            hexes.push(
                <Hex
                    key={key}
                    _key={key}
                    onClick={props.handleCellClick}
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

    return (
        <svg version="1.1"
            baseProfile="full"
            xmlns="http://www.w3.org/2000/svg"
            className="board"
            viewBox={`0 0 ${size} ${size}`}>
            {hexes}
        </svg>
    );
});

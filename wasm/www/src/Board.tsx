import * as React from "react";

import Hex from "./Hex";

import GameState from "./GameState";

import { NUM_ROWS, EVEN_ROW_LEN, ODD_ROW_LEN, HUMAN_PLAYER, BOT_PLAYER } from "./constants";

type Props = {
    gameState: GameState,
    possibleMoves: number[],
    chosenCell?: number,
    handleCellClick: ((idx: number) => void),
};

const Board = React.memo(function (props: Props) {
    const size = 1000;
    const side_length = size / 16;

    const start_x = 2 * side_length;
    const start_y = 2 * side_length;

    const players: Map<number, number> = new Map();
    for (const p of [HUMAN_PLAYER, BOT_PLAYER]) {
        for (const c of props.gameState.board.penguins[p] || []) {
            players.set(c, p);
        }
    }

    const any_possible_moves = props.possibleMoves.length > 0;

    const claimed: Set<number> = new Set([]);
    for (const player_claimed of props.gameState.board.claimed) {
        for (const cell of player_claimed) {
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
            const fish = props.gameState.board.fish[key] || 0;
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
                    player={players.get(key)}
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

export default Board;
import * as React from "react";

import Hex from "./Hex";
import { type GameState, type MoveScore } from "./WorkerProtocol";
import {
  NUM_ROWS,
  EVEN_ROW_LEN,
  ODD_ROW_LEN,
  HUMAN_PLAYER,
  BOT_PLAYER,
} from "./constants";

interface Props {
  gameState: GameState;
  possibleMoves: number[];
  chosenCell?: number;
  handleCellClick: (idx: number) => void;
  topMove?: MoveScore;
}

export default React.memo(function Board(props: Props): React.JSX.Element {
  const size = 1000;
  const sideLength = size / 16;

  const startX = 2 * sideLength;
  const startY = 2 * sideLength;

  const players = new Map<number, number>();
  for (const p of [HUMAN_PLAYER, BOT_PLAYER]) {
    for (const c of props.gameState.board.penguins[p] ?? []) {
      players.set(c, p);
    }
  }

  const anyPossibleMoves = props.possibleMoves.length > 0;

  const claimed = new Set<number>([]);
  for (const playerClaimed of props.gameState.board.claimed) {
    for (const cell of playerClaimed) {
      claimed.add(cell);
    }
  }

  const hexes = [];
  for (let r = 0; r < NUM_ROWS; ++r) {
    const y = startY + r * 1.5 * sideLength;

    const rowLength = r % 2 === 0 ? EVEN_ROW_LEN : ODD_ROW_LEN;
    const xBobble = (-1 * (r % 2) * Math.sqrt(3) * sideLength) / 2;

    for (let c = 0; c < rowLength; ++c) {
      const hexWidth =
        sideLength * (Math.sin(Math.PI / 3) - Math.sin((Math.PI * 5) / 3));
      const x = startX + c * hexWidth + xBobble;

      const key = hexes.length;
      const fish = props.gameState.board.fish[key] ?? 0;
      const possible = props.possibleMoves.includes(key);

      const isHighlighted = anyPossibleMoves && props.chosenCell === key;

      hexes.push(
        <Hex
          key={key}
          _key={key}
          onClick={props.handleCellClick}
          fish={fish}
          cx={x}
          cy={y}
          sideLength={sideLength}
          highlighted={isHighlighted}
          possible={possible}
          player={players.get(key)}
          claimed={claimed.has(key)}
          isTopMoveSrc={props.topMove?.src === key}
          isTopMoveDst={props.topMove?.dst === key}
        />,
      );
    }
  }

  return (
    <svg
      version="1.1"
      baseProfile="full"
      xmlns="http://www.w3.org/2000/svg"
      className="board"
      viewBox={`0 0 ${size} ${size}`}
    >
      {hexes}
    </svg>
  );
});

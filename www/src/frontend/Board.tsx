import * as React from "react";

import { type GameState, type MoveScore } from "../browser/WorkerProtocol";
import {
  BOT_PLAYER,
  EVEN_ROW_LEN,
  HUMAN_PLAYER,
  NUM_ROWS,
  ODD_ROW_LEN,
} from "../browser/constants";
import Hex from "./Hex";

import "./index.css";

interface Props {
  gameState: GameState;
  possibleMoves: number[];
  chosenCell?: number;
  handleCellClick: (idx: number) => void;
  botMoveScores?: MoveScore[];
}

/**
 * HSL hues, one per penguin
 */
const SOURCE_PENGUIN_HUES = [
  // green
  120,
  // blue
  200,
  // orange
  30,
  // purple
  280,
] as const;

function computeHeatmap(botMoveScores: MoveScore[]): {
  cellFills: Map<number, string>;
  sourcePenguinFills: Map<number, string>;
} {
  const cellFills = new Map<number, string>();
  const sourcePenguinFills = new Map<number, string>();

  // Assign a unique hue to each source penguin
  const sourcePenguins = [
    ...new Set(
      botMoveScores.map((s) => s.src).filter((src) => src !== undefined),
    ),
  ];
  const sourceHues = new Map<number, number>();
  sourcePenguins.forEach((src, idx) => {
    const hue = SOURCE_PENGUIN_HUES[idx];
    if (hue) {
      sourceHues.set(src, hue);
      sourcePenguinFills.set(src, `hsl(${hue.toString()}, 80%, 80%)`);
    }
  });

  // Calculate total visits for normalization
  let totalVisits = 0;
  for (const score of botMoveScores) {
    totalVisits += score.visits;
  }

  // Compute fill colors for destination cells
  if (totalVisits > 0) {
    for (const score of botMoveScores) {
      const intensity = score.visits / totalVisits;
      const hue =
        score.src !== undefined ? (sourceHues.get(score.src) ?? 120) : 120;
      const saturation = Math.round(intensity * 100);
      const lightness = 50 + Math.round((1 - intensity) * 40); // 50-90% lightness
      cellFills.set(
        score.dst,
        `hsl(${hue.toString()}, ${saturation.toString()}%, ${lightness.toString()}%)`,
      );
    }
  }

  return { cellFills, sourcePenguinFills };
}

export default function Board(props: Props): React.JSX.Element {
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

  // Compute heatmap fills if bot is thinking
  const { cellFills, sourcePenguinFills } = props.botMoveScores
    ? computeHeatmap(props.botMoveScores)
    : {};

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

      const fillColor = cellFills?.get(key) ?? sourcePenguinFills?.get(key);

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
          fillColor={fillColor}
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
      data-testid="board"
      viewBox={`0 0 ${size.toString()} ${size.toString()}`}
    >
      {hexes}
    </svg>
  );
}

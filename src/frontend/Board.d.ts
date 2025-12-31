import * as React from "react";
import { type GameState, type MoveScore } from "../browser/WorkerProtocol";
import "./index.css";
interface Props {
    gameState: GameState;
    possibleMoves: number[];
    chosenCell?: number;
    handleCellClick: (idx: number) => void;
    botMoveScores?: MoveScore[];
}
export default function Board(props: Props): React.JSX.Element;
export {};

import * as wasm from "htmf-wasm";
import { type GameState, type MoveScore, type WorkerRequest, type WorkerResponse } from "../browser/WorkerProtocol";
declare class Bot {
    wasmInternals: wasm.InitOutput;
    game: wasm.Game;
    postMessage: (msg: WorkerResponse) => void;
    ponderer?: number;
    forcedMove: boolean;
    ponderStartTime?: number;
    totalCompletedPonderTimeMs: number;
    constructor(wasmInternals: wasm.InitOutput, postMessage: (msg: WorkerResponse) => void);
    free(): void;
    ponder(): void;
    moveNow(): void;
    stopPondering(): void;
    placePenguin(dst: number): void;
    movePenguin(src: number, dst: number): void;
    playout(): void;
    getState(): GameState;
    getPossibleMoves(src?: number): number[];
    onMessage(request: WorkerRequest): void;
    postGameState({ src, lastMoveWasIllegal, }: {
        src?: number;
        lastMoveWasIllegal?: boolean;
    }): void;
    postThinkingProgress(): void;
    getMoveScores(activePlayer: number): MoveScore[];
    getTotalTimeThinkingMs(): number;
}
export default Bot;

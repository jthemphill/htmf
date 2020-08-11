import GameState from "./GameState";

type WorkerRequest = {
    type: "get",
} | {
    type: "place",
    dst: number,
} | {
    type: "move",
    src: number,
    dst: number,
} | {
    type: "possibleMoves",
    src?: number,
} | {
    type: "takeAction",
} | {
    type: "startPondering",
} | {
    type: "takeAction",
};

type WorkerResponse = {
    type: "state",
    gameState: GameState,
} | {
    type: "possibleMoves",
    possibleMoves: number[],
} | {
    type: "illegalMove",
    src: number,
    dst: number,
} | {
    type: "illegalPlacement",
    dst: number,
};

export { WorkerRequest, WorkerResponse };
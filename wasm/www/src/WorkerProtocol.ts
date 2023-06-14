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
    type: "initialized",
    gameState: GameState,
    possibleMoves: number[],
} | {
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
} | {
    type: "placeScores",
    activePlayer: number,
    placeScores: {
        dst: number,
        visits: number,
        rewards: number,
    }[],
} | {
    type: "moveScores",
    activePlayer: number,
    moveScores: {
        src: number,
        dst: number,
        visits: number,
        rewards: number,
    }[],
} | {
    type: "thinkingProgress",
    completed: number,
    required: number,
};

export { WorkerRequest, WorkerResponse };
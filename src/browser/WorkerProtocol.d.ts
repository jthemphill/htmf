export interface GameState {
    activePlayer?: number;
    modeType: "drafting" | "playing";
    scores: number[];
    turn: number;
    board: {
        fish: number[];
        penguins: number[][];
        claimed: number[][];
    };
}
export interface MoveScore {
    src?: number;
    dst: number;
    visits: number;
    rewards: number;
}
export interface BotWorker extends Worker {
    onmessage: (event: MessageEvent<WorkerResponse>) => void;
    postMessage: (request: WorkerRequest) => void;
}
export interface PlayerMoveScores {
    player: number;
    moveScores: MoveScore[];
}
export type WorkerRequest = {
    type: "getGameState";
} | {
    type: "getPossibleMoves";
    src?: number;
} | {
    type: "movePenguin";
    src?: number;
    dst: number;
};
export type WorkerResponse = {
    type: "initialized";
    worker: BotWorker;
} | {
    type: "gameState";
    gameState: GameState;
    possibleMoves: number[];
    lastMoveWasIllegal: boolean;
} | {
    type: "thinkingProgress";
    playerMoveScores: PlayerMoveScores;
    memoryUsage: number;
    treeSize: number;
    completed: number;
    required: number;
    totalPlayouts: number;
    totalTimeThinkingMs: number;
} | {
    type: "terminated";
};

import * as wasm from "htmf-wasm";
import { BOT_PLAYER, HUMAN_PLAYER, MAX_PLAYOUTS, MIN_PLAYOUTS, NPLAYERS, NUM_CELLS, PLAYOUT_CHUNK_SIZE, } from "../browser/constants";
function getGameState(game) {
    const fish = [];
    for (let idx = 0; idx < NUM_CELLS; ++idx) {
        fish.push(game.num_fish(idx));
    }
    const scores = [];
    const penguins = [];
    const claimed = [];
    for (let p = 0; p < NPLAYERS; ++p) {
        scores.push(game.score(p));
        penguins.push([...game.penguins(p)]);
        claimed.push([...game.claimed(p)]);
    }
    return {
        activePlayer: game.active_player(),
        modeType: game.finished_drafting() ? "playing" : "drafting",
        scores,
        turn: game.turn(),
        board: {
            fish,
            penguins,
            claimed,
        },
    };
}
function getPossibleMoves(game, src) {
    if (game.active_player() !== HUMAN_PLAYER) {
        return [];
    }
    if (game.finished_drafting()) {
        if (src !== undefined) {
            return [...game.possible_moves(src)];
        }
        const activePlayer = game.active_player();
        return activePlayer === undefined ? [] : [...game.penguins(activePlayer)];
    }
    else {
        return [...game.draftable_cells()];
    }
}
class Bot {
    wasmInternals;
    game = wasm.Game.new();
    postMessage;
    ponderer;
    forcedMove = false;
    ponderStartTime;
    totalCompletedPonderTimeMs = 0;
    constructor(wasmInternals, postMessage) {
        this.wasmInternals = wasmInternals;
        this.postMessage = postMessage;
        this.ponder();
        this.postGameState({});
    }
    free() {
        this.stopPondering();
        this.game.free();
        this.totalCompletedPonderTimeMs = 0;
    }
    ponder() {
        if (this.ponderer !== undefined) {
            return;
        }
        this.ponderStartTime = performance.now();
        this.ponderer = self.setInterval(() => {
            const activePlayer = this.game.active_player();
            if (activePlayer === BOT_PLAYER) {
                // We need to make a move soon
                if (this.forcedMove || this.game.get_visits() >= MIN_PLAYOUTS) {
                    // Move if the human player forced us or if we clear the minimum threshold
                    this.game.take_action();
                    this.forcedMove = false;
                    this.postGameState({});
                    this.ponder();
                }
            }
            else if (activePlayer === undefined ||
                this.game.get_visits() >= MAX_PLAYOUTS) {
                // The game is over or we've consumed our playout budget; stop thinking
                this.stopPondering();
            }
            this.game.playout_n_times(PLAYOUT_CHUNK_SIZE);
            this.postThinkingProgress();
        });
    }
    moveNow() {
        this.forcedMove = true;
        if (this.game.active_player() === BOT_PLAYER) {
            this.ponder();
        }
    }
    stopPondering() {
        if (this.ponderer !== undefined) {
            clearInterval(this.ponderer);
            this.ponderer = undefined;
        }
        if (this.ponderStartTime !== undefined) {
            this.totalCompletedPonderTimeMs +=
                performance.now() - this.ponderStartTime;
            this.ponderStartTime = undefined;
        }
        this.postThinkingProgress();
    }
    placePenguin(dst) {
        this.game.place_penguin(dst);
        this.ponder();
    }
    movePenguin(src, dst) {
        this.game.move_penguin(src, dst);
        this.ponder();
    }
    playout() {
        this.game.playout();
    }
    getState() {
        return getGameState(this.game);
    }
    getPossibleMoves(src) {
        return getPossibleMoves(this.game, src);
    }
    onMessage(request) {
        console.log(`received request ${request.type}`);
        let src;
        let lastMoveWasIllegal = false;
        switch (request.type) {
            case "getGameState":
                break;
            case "movePenguin":
                try {
                    if (request.src === undefined) {
                        this.placePenguin(request.dst);
                    }
                    else {
                        this.movePenguin(request.src, request.dst);
                    }
                }
                catch (err) {
                    void err;
                    lastMoveWasIllegal = true;
                }
                break;
            case "getPossibleMoves":
                src = request.src;
                break;
            case "pausePondering":
                this.stopPondering();
                return;
            case "resumePondering":
                this.ponder();
                return;
            case "moveNow":
                this.moveNow();
                return;
        }
        this.postGameState({ src, lastMoveWasIllegal });
    }
    postGameState({ src, lastMoveWasIllegal, }) {
        lastMoveWasIllegal = lastMoveWasIllegal === true;
        const postMessage = this.postMessage;
        postMessage({
            type: "gameState",
            gameState: this.getState(),
            possibleMoves: this.getPossibleMoves(src),
            lastMoveWasIllegal,
        });
    }
    postThinkingProgress() {
        const activePlayer = this.game.active_player();
        const postMessage = this.postMessage;
        postMessage({
            type: "thinkingProgress",
            completed: this.game.get_visits(),
            required: activePlayer === BOT_PLAYER ? MIN_PLAYOUTS : MAX_PLAYOUTS,
            totalPlayouts: this.game.get_total_playouts(),
            totalTimeThinkingMs: this.getTotalTimeThinkingMs(),
            memoryUsage: this.wasmInternals.memory.buffer.byteLength,
            isPondering: this.ponderer !== undefined,
            treeSize: this.game.tree_size(),
            playerMoveScores: activePlayer !== undefined
                ? {
                    player: activePlayer,
                    moveScores: this.getMoveScores(activePlayer),
                }
                : {},
        });
    }
    getMoveScores(activePlayer) {
        const moveScores = [];
        if (this.game.is_drafting()) {
            for (const dst of this.game.draftable_cells()) {
                const info = this.game.place_info(dst);
                moveScores.push({
                    dst,
                    visits: info.get_visits(),
                    rewards: info.get_rewards(),
                });
            }
        }
        else {
            for (const src of this.game.penguins(activePlayer)) {
                for (const dst of this.game.possible_moves(src)) {
                    const info = this.game.move_info(src, dst);
                    moveScores.push({
                        src,
                        dst,
                        visits: info.get_visits(),
                        rewards: info.get_rewards(),
                    });
                }
            }
        }
        return moveScores;
    }
    getTotalTimeThinkingMs() {
        let totalTimeThinkingMs = this.totalCompletedPonderTimeMs;
        if (this.ponderStartTime !== undefined) {
            totalTimeThinkingMs += performance.now() - this.ponderStartTime;
        }
        return totalTimeThinkingMs;
    }
}
export default Bot;

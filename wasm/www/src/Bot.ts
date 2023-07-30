import * as wasm from "htmf-wasm";

import { NPLAYERS, NUM_CELLS, PLAYOUT_MS, PONDER_INTERVAL_MS, MIN_PLAYOUTS, MAX_PLAYOUTS, HUMAN_PLAYER } from "./constants";
import GameState from "./GameState";
import { WorkerRequest, WorkerResponse } from "./WorkerProtocol";

function getGameState(game: wasm.Game): GameState {
    const fish = [];
    for (let idx = 0; idx < NUM_CELLS; ++idx) {
        fish.push(game.num_fish(idx));
    }
    const scores: number[] = [];
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
        }
    };
}

function getPossibleMoves(game: wasm.Game, src?: number): number[] {
    if (game.active_player() !== HUMAN_PLAYER) {
        return [];
    }
    if (game.finished_drafting()) {
        if (src !== undefined) {
            return [...game.possible_moves(src)];
        }
        const active_player = game.active_player();
        return active_player === undefined ? [] : [...game.penguins(active_player)];
    } else {
        return [...game.draftable_cells()];
    }
}

class Bot {
    game: wasm.Game;
    postMessage: (msg: WorkerResponse) => void
    ponderer?: number;
    nplayouts = 0;

    constructor(postMessage: (msg: WorkerResponse) => void) {
        this.game = wasm.Game.new();
        this.postMessage = postMessage;
        this.ponder();
    }

    free() {
        this.stopPondering();
        this.game.free();
    }

    init() {
        const postMessage = this.postMessage;
        postMessage({
            type: "initialized",
            gameState: this.getState(),
            possibleMoves: this.getPossibleMoves(),
        });
    }

    ponder() {
        this.nplayouts = this.game.get_visits();

        if (this.ponderer !== undefined) {
            return;
        }
        this.ponderer = setInterval(
            () => {
                if (this.nplayouts >= MAX_PLAYOUTS) {
                    this.stopPondering();
                    return;
                }
                const t0 = performance.now();
                while (performance.now() - t0 < PLAYOUT_MS) {
                    this.playout();
                }

                const activePlayer = this.game.active_player();
                if (activePlayer !== undefined) {
                    if (this.game.is_drafting()) {
                        this.postPlaceScores(activePlayer);
                    } else {
                        this.postMoveScores(activePlayer);
                    }
                }
            },
            PONDER_INTERVAL_MS,
        );
    }

    stopPondering() {
        if (this.ponderer !== undefined) {
            clearInterval(this.ponderer);
            this.ponderer = undefined;
        }
    }

    placePenguin(dst: number) {
        this.game.place_penguin(dst);
        this.ponder();
    }

    movePenguin(src: number, dst: number) {
        this.game.move_penguin(src, dst);
        this.ponder();
    }

    playout() {
        this.game.playout();
        ++this.nplayouts;
    }

    takeAction() {
        this.stopPondering();
        const postMessage = this.postMessage;
        const min_playouts = this.game.turn() < 2 ? 2 * MIN_PLAYOUTS : MIN_PLAYOUTS;
        while (this.nplayouts < min_playouts) {
            this.playout();
            if (this.nplayouts % 100 === 0) {
                postMessage({
                    type: "thinkingProgress",
                    completed: this.nplayouts,
                    required: min_playouts,
                });
            }
        }
        this.game.take_action();
        this.ponder();
    }

    getState(): GameState {
        return getGameState(this.game);
    }

    getPossibleMoves(src?: number): number[] {
        return getPossibleMoves(this.game, src);
    }

    onMessage(request: WorkerRequest) {
        console.log(`received request ${request.type}`);
        switch (request.type) {
            case "get":
                this.postGameState();
                break;
            case "move":
                try {
                    this.movePenguin(request.src, request.dst);
                    this.postGameState();
                } catch (err) {
                    this.postIllegalMove(request.src, request.dst);
                }
                break;
            case "place":
                try {
                    this.placePenguin(request.dst);
                    this.postGameState();
                } catch (err) {
                    this.postIllegalPlacement(request.dst);
                }
                break;
            case "possibleMoves":
                this.postPossibleMoves(request.src);
                break;
            case "takeAction":
                this.takeAction();
                this.postGameState();
                break;
        }
    }

    postIllegalMove(src: number, dst: number) {
        const postMessage = this.postMessage;
        postMessage({
            type: "illegalMove",
            src,
            dst,
        });
    }

    postIllegalPlacement(dst: number) {
        const postMessage = this.postMessage;
        postMessage({
            type: "illegalPlacement",
            dst,
        });
    }

    postPossibleMoves(src?: number) {
        const postMessage = this.postMessage;
        postMessage({
            type: "possibleMoves",
            possibleMoves: this.getPossibleMoves(src),
        });
    }

    postGameState() {
        const postMessage = this.postMessage;
        postMessage({
            type: "state",
            gameState: this.getState(),
        });
    }

    postPlaceScores(activePlayer: number) {
        const placeScores = [];
        for (const dst of this.game.draftable_cells()) {
            const info = this.game.place_info(dst);
            placeScores.push({
                dst,
                visits: info.get_visits(),
                rewards: info.get_rewards(),
            });
        }
        const postMessage = this.postMessage;
        postMessage({
            type: "placeScores",
            activePlayer,
            placeScores,
        });
    }

    postMoveScores(activePlayer: number) {
        const moveScores = [];
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
        const postMessage = this.postMessage;
        postMessage({
            type: "moveScores",
            activePlayer,
            moveScores,
        });
    }
}

export default Bot;
"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    Object.defineProperty(o, k2, { enumerable: true, get: function() { return m[k]; } });
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
const wasm = __importStar(require("htmf-wasm"));
const constants_1 = require("./constants");
function getGameState(game) {
    const fish = [];
    for (let idx = 0; idx < constants_1.NUM_CELLS; ++idx) {
        fish.push(game.num_fish(idx));
    }
    const scores = [];
    const penguins = [];
    const claimed = [];
    for (let p = 0; p < constants_1.NPLAYERS; ++p) {
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
function getPossibleMoves(game, src) {
    if (game.active_player() != constants_1.HUMAN_PLAYER) {
        return [];
    }
    if (game.finished_drafting()) {
        if (src != null) {
            return [...game.possible_moves(src)];
        }
        const active_player = game.active_player();
        return active_player == null ? [] : [...game.penguins(active_player)];
    }
    else {
        return [...game.draftable_cells()];
    }
}
class Bot {
    constructor(postMessage) {
        this.game = wasm.Game.new();
        this.postMessage = postMessage;
        this.nplayouts = 0;
    }
    free() {
        this.stopPondering();
        this.game.free();
    }
    init() {
        const postMessage = this.postMessage;
        postMessage({
            type: "state",
            gameState: this.getState(),
        });
        postMessage({
            type: "possibleMoves",
            possibleMoves: this.getPossibleMoves(),
        });
    }
    ponder() {
        this.nplayouts = 0;
        if (this.ponderer != null) {
            return;
        }
        this.ponderer = self.setInterval(() => {
            if (this.nplayouts >= constants_1.MAX_PLAYOUTS) {
                this.stopPondering();
                return;
            }
            const t0 = performance.now();
            while (performance.now() - t0 < constants_1.PLAYOUT_MS) {
                this.playout();
            }
            // console.log(`Finished ${nplayouts} in ${performance.now() - t0} ms.`);
        }, constants_1.PONDER_INTERVAL_MS);
    }
    stopPondering() {
        if (this.ponderer != undefined) {
            self.clearInterval(this.ponderer);
            this.ponderer = undefined;
        }
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
        ++this.nplayouts;
    }
    takeAction() {
        this.stopPondering();
        while (this.nplayouts < constants_1.MAX_PLAYOUTS) {
            this.playout();
        }
        this.game.take_action();
        this.ponder();
    }
    getState() {
        return getGameState(this.game);
    }
    getPossibleMoves(src) {
        return getPossibleMoves(this.game, src);
    }
    onMessage(request) {
        console.log(`received request ${request.type}`);
        switch (request.type) {
            case "get":
                this.postGameState();
                break;
            case "move":
                try {
                    this.movePenguin(request.src, request.dst);
                    this.postGameState();
                }
                catch (err) {
                    this.postIllegalMove(request.src, request.dst);
                }
                break;
            case "place":
                try {
                    this.placePenguin(request.dst);
                    this.postGameState();
                }
                catch (err) {
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
    postIllegalMove(src, dst) {
        const postMessage = this.postMessage;
        postMessage({
            type: "illegalMove",
            src,
            dst,
        });
    }
    postIllegalPlacement(dst) {
        const postMessage = this.postMessage;
        postMessage({
            type: "illegalPlacement",
            dst,
        });
    }
    postPossibleMoves(src) {
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
}
exports.default = Bot;
//# sourceMappingURL=Bot.js.map
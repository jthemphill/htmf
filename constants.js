"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.PONDER_INTERVAL_MS = exports.MAX_PLAYOUTS = exports.PLAYOUT_MS = exports.PLAYER_COLORS = exports.NUM_CELLS = exports.ODD_ROW_LEN = exports.EVEN_ROW_LEN = exports.NUM_ROWS = exports.BOT_PLAYER = exports.HUMAN_PLAYER = exports.NPLAYERS = void 0;
const NPLAYERS = 2;
exports.NPLAYERS = NPLAYERS;
const HUMAN_PLAYER = 0;
exports.HUMAN_PLAYER = HUMAN_PLAYER;
const BOT_PLAYER = 1;
exports.BOT_PLAYER = BOT_PLAYER;
const NUM_ROWS = 8;
exports.NUM_ROWS = NUM_ROWS;
const EVEN_ROW_LEN = 7;
exports.EVEN_ROW_LEN = EVEN_ROW_LEN;
const ODD_ROW_LEN = 8;
exports.ODD_ROW_LEN = ODD_ROW_LEN;
const NUM_CELLS = EVEN_ROW_LEN * (NUM_ROWS / 2) + ODD_ROW_LEN * (NUM_ROWS / 2);
exports.NUM_CELLS = NUM_CELLS;
const PLAYOUT_MS = 100;
exports.PLAYOUT_MS = PLAYOUT_MS;
const PONDER_INTERVAL_MS = 0;
exports.PONDER_INTERVAL_MS = PONDER_INTERVAL_MS;
const MAX_PLAYOUTS = 6000;
exports.MAX_PLAYOUTS = MAX_PLAYOUTS;
// duplicated in board.react.js
const PLAYER_COLORS = ['blue', 'red', 'orange', 'green'];
exports.PLAYER_COLORS = PLAYER_COLORS;
//# sourceMappingURL=constants.js.map
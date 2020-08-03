const NPLAYERS = 2;
const NUM_ROWS = 8;
const EVEN_ROW_LEN = 7;
const ODD_ROW_LEN = 8;
const NUM_CELLS = EVEN_ROW_LEN * (NUM_ROWS / 2) + ODD_ROW_LEN * (NUM_ROWS / 2);

const PLAYOUT_MS = 100;
const PONDER_INTERVAL_MS = 1000;

// duplicated in board.react.js
const PLAYER_COLORS = ['blue', 'red', 'orange', 'green'];

export {
    NPLAYERS,
    NUM_ROWS,
    EVEN_ROW_LEN,
    ODD_ROW_LEN,
    NUM_CELLS,
    PLAYER_COLORS,
    PLAYOUT_MS,
    PONDER_INTERVAL_MS
};
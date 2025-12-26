export const NPLAYERS = 2;
export const HUMAN_PLAYER = 0;
export const BOT_PLAYER = 1;
export const NUM_ROWS = 8;
export const EVEN_ROW_LEN = 7;
export const ODD_ROW_LEN = 8;
export const NUM_CELLS: number =
  EVEN_ROW_LEN * (NUM_ROWS / 2) + ODD_ROW_LEN * (NUM_ROWS / 2);

export const PLAYOUT_CHUNK_SIZE = 200;
export const MIN_PLAYOUTS = 28_000;
export const MAX_PLAYOUTS = 120_000;

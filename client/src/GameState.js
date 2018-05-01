type State = {
    last_move_valid: bool,
    mode_type: string,
    nplayers: number,
    active_player: number,
    scores: Array<number>,
    turn: number,
    board: {
        fish: Array<number>,
        penguins: Array<Array<number>>,
        live: Array<number>,
        possible_moves: Array<bool>,
    },
};

export default State;

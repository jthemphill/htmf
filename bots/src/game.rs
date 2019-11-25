extern crate mcts;

extern crate htmf;

#[derive(Clone)]
struct Game {
    state: htmf::game::GameState,
}

#[derive(Clone, Copy)]
enum Move {
    Place(u8),
    Move((u8, u8)),
}

impl self::mcts::GameState for Game {
    type Move = Move;
    type Player = htmf::board::Player;
    type MoveList = Vec<Move>;

    fn current_player(&self) -> Self::Player {
        self.state.active_player().unwrap()
    }

    fn available_moves(&self) -> Vec<Self::Move> {
        if self.state.game_over() {
            return vec![];
        }
        let p = self.current_player();
        if self.state.finished_drafting() {
            all_moves(&self.state, p)
        } else {
            // Cells with one fish and nobody claiming them
            let mut draftable_cells = self.state.board.fish[0].clone();
            draftable_cells.exclude(&self.state.board.all_claimed_cells());
            draftable_cells
                .iter()
                .map(|cell| Move::Place(cell))
                .collect()
        }
    }

    fn make_move(&mut self, mov: &Self::Move) {
        match mov {
            Move::Place(dst) => self.state.place_penguin(*dst).unwrap(),
            Move::Move((src, dst)) => self.state.move_penguin(*src, *dst).unwrap(),
        }
    }
}

fn all_moves(game: &htmf::game::GameState, p: htmf::board::Player) -> Vec<Move> {
    game.board.penguins[p.id]
        .into_iter()
        .flat_map(|src| {
            let move_vec: Vec<Move> = game
                .board
                .moves(src)
                .into_iter()
                .map(|dst| Move::Move((src, dst)))
                .collect();
            move_vec.into_iter()
        })
        .collect()
}

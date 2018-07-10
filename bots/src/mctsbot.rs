use mcts::*;
use mcts::tree_policy::*;

extern crate htmf;

struct MyMCTS;

struct MyEvaluator;

type MoveVec = Vec<Move>;

impl Evaluator<MyMCTS> for MyEvaluator {
    type StateEvaluation =i32;

    fn evaluate_new_state(&self, game: &Game, moves: &MoveVec,
        _: Option<SearchHandle<MyMCTS>>)
        -> (Vec<()>, Self::StateEvaluation) {
        let scores = game.state.get_scores();
        (vec![(); moves.len()], scores[1] as i32 - scores[0] as i32)
    }
    fn interpret_evaluation_for_player(&self, evaln: &Self::StateEvaluation, player: &htmf::board::Player) -> i64 {
        if player.id == 1 {
            *evaln as i64
        } else {
            -*evaln as i64
        }
    }
    fn evaluate_existing_state(&self, _game: &Game, evaln: &Self::StateEvaluation, _: SearchHandle<MyMCTS>) -> Self::StateEvaluation {
        *evaln
    }
}

impl MCTS for MyMCTS {
    type State = Game;
    type Eval = MyEvaluator;
    type NodeData = ();
    type ExtraThreadData = ();
    type TreePolicy = UCTPolicy;
}

#[derive(Clone)]
pub struct Game {
    state: htmf::game::GameState,
}

#[derive(Clone, Copy)]
pub enum Move {
    Place(u8),
    Move((u8, u8)),
}

impl GameState for Game {
    type Move = Move;
    type Player = htmf::board::Player;
    type MoveList = MoveVec;

    fn current_player(&self) -> Self::Player {
        self.state.active_player().unwrap()
    }

    fn available_moves(&self) -> Self::MoveList {
        if self.state.game_over() {
            return Vec::new();
        }
        let p = self.current_player();
        if self.state.finished_drafting() {
            all_moves(&self.state, p)
        } else {
            // Cells with one fish and nobody claiming them
            let mut draftable_cells = self.state.board.fish[0].clone();
            draftable_cells.exclude(&self.state.board.all_claimed_cells());
            draftable_cells.iter().map(|cell| Move::Place(cell)).collect()
        }
    }

    fn make_move(&mut self, mov: &Self::Move) {
        match mov {
            Move::Place(dst) => self.state.place_penguin(*dst).unwrap(),
            Move::Move((src, dst)) => {
                let p = self.state.active_player().unwrap();
                self.state.board.claimed[p.id].insert(*dst);
                self.state.board.penguins[p.id].remove(*src);
                self.state.board.penguins[p.id].insert(*dst);
            },
        }
    }
}

fn all_moves(game: &htmf::game::GameState, p: htmf::board::Player) -> MoveVec {
    let mut moves = Vec::new();
    for src in game.board.penguins[p.id].into_iter() {
        for dst in game.board.moves(src).into_iter() {
            moves.push(Move::Move((src, dst)));
        }
    }
    moves
}

#[derive(Clone)]
pub struct MCTSBot {
    pub game: Game,
    pub me: htmf::board::Player,
}

impl MCTSBot {
    pub fn new(game: &htmf::game::GameState, me: htmf::board::Player) -> Self {
        MCTSBot {
            game: Game{state: game.clone()},
            me,
        }
    }

    pub fn update(&mut self, game: &htmf::game::GameState) {
        self.game = Game{state: game.clone()};
    }

    pub fn take_action(&mut self) -> htmf::game::Action {
        if self.game.state.active_player() != Some(self.me) {
            panic!("{:?} was asked to move, but it is not their turn!", self.me);
        }
        let mut mcts = MCTSManager::new(self.game.clone(), MyMCTS, MyEvaluator, UCTPolicy::new(0.5));
        mcts.playout_n(100);
        let pv = mcts.principal_variation(1);
        if pv.len() == 0 {
            panic!(format!("No moves??? {}...", self.game.state.game_over()));
        }
        let best_move = pv.into_iter().next().unwrap();
        match best_move {
            Move::Move((src, dst)) => htmf::game::Action::Move(src, dst),
            Move::Place(dst) => htmf::game::Action::Place(dst)
        }
    }
}

extern crate itertools;
extern crate pathfinding;

use arrayvec::ArrayVec;
use rand::{Rng, SeedableRng, StdRng};

use cellset::CellSet;
use {EVEN_ROW_LEN, NUM_CELLS, NUM_ONE_FISH, NUM_THREE_FISH, NUM_TWO_FISH, ODD_ROW_LEN};

use self::itertools::Itertools;

use errors::IllegalMoveError;
use hex::{line, EvenR};

type Fish = usize;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Player {
    pub id: usize,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Board {
    pub penguins: ArrayVec<[CellSet; 4]>,
    pub fish: ArrayVec<[CellSet; 3]>,
    pub claimed: ArrayVec<[CellSet; 4]>,
}

impl Board {
    pub fn new(seed: [u8; 32]) -> Board {
        let mut cell_to_fish: [Fish; NUM_CELLS] = [1; NUM_CELLS];
        let mut size: usize = NUM_ONE_FISH;

        for _ in 0..NUM_TWO_FISH {
            cell_to_fish[size] = 2;
            size += 1
        }
        for _ in 0..NUM_THREE_FISH {
            cell_to_fish[size] = 3;
            size += 1;
        }

        {
            let mut rng: StdRng = SeedableRng::from_seed(seed);
            rng.shuffle(&mut cell_to_fish);
        }

        let mut fish: ArrayVec<[CellSet; 3]> = ArrayVec::new();
        for _ in 0..3 {
            fish.push(CellSet::new());
        }
        for i in 0..NUM_CELLS {
            fish[cell_to_fish[i] - 1].insert(i as u8);
        }

        Board {
            fish,
            penguins: (0..4).map(|_| CellSet::new()).collect(),
            claimed: (0..4).map(|_| CellSet::new()).collect(),
        }
    }

    pub fn num_fish(&self, idx: u8) -> usize {
        self.fish
            .iter()
            .enumerate()
            .filter(|&(_, cells_with_fish)| cells_with_fish.contains(idx))
            .nth(0)
            .unwrap()
            .0
            + 1
    }

    pub fn all_claimed_cells(&self) -> CellSet {
        let mut ret = CellSet::new();
        for cells in self.claimed.iter() {
            ret.union(cells);
        }
        ret
    }

    pub fn is_claimed(&self, idx: u8) -> bool {
        self.claimed.iter().any(|cells| cells.contains(idx as u8))
    }

    pub fn claim_cell(&mut self, p: Player, idx: u8) -> Result<(), IllegalMoveError> {
        if self.is_claimed(idx) {
            return Err(IllegalMoveError::new(
                p,
                format!("Cell at {} already claimed", idx),
            ));
        }

        self.claimed[p.id].insert(idx);
        self.penguins[p.id].insert(idx as u8);
        Ok(())
    }

    pub fn get_score(&self, p: Player) -> usize {
        (1..=3)
            .into_iter()
            .map(|num_fish| {
                let mut claimed_with_fish = self.claimed[p.id].clone();
                claimed_with_fish.intersect(&self.fish[num_fish - 1]);
                claimed_with_fish.len() * num_fish
            })
            .sum()
    }

    pub fn move_penguin(&mut self, p: Player, src: u8, dst: u8) -> Result<&Self, IllegalMoveError> {
        if !self.is_legal_move(p, src, dst) {
            return Err(IllegalMoveError::new(
                p,
                format!(
                    "Player {:?} cannot move penguin from {:?} to {:?}",
                    p, src, dst
                ),
            ));
        }
        self.claimed[p.id].insert(dst);
        self.penguins[p.id].remove(src as u8);
        self.penguins[p.id].insert(dst as u8);
        Ok(self)
    }

    fn neighbors(c: u8) -> impl Iterator<Item = u8> {
        Board::index_to_evenr(c)
            .neighbors()
            .into_iter()
            .filter(|neighbor| Board::in_bounds(neighbor))
            .map(|evenr| Board::evenr_to_index(&evenr))
    }

    fn in_bounds(c: &EvenR) -> bool {
        c.row >= 0 && Board::in_column_bounds(c) && Board::evenr_to_index(c) < NUM_CELLS as u8
    }

    fn in_column_bounds(c: &EvenR) -> bool {
        if c.col < 0 {
            false
        } else if c.row & 1 == 0 {
            c.col < EVEN_ROW_LEN as i64
        } else {
            c.col < ODD_ROW_LEN as i64
        }
    }

    pub fn claimed_cells(&self) -> CellSet {
        let mut ret = CellSet::new();
        for cells in self.claimed.iter() {
            ret.union(cells);
        }
        ret
    }

    /// Is the given cell a "cut cell"?
    ///
    /// Look at each of the cell's unclaimed neighbors and return true
    /// if there are any claimed cells separating them.
    ///
    /// This is designed to be a cheap check to figure out if moving a
    /// penguin from the given cell cannot possibly increase the
    /// number of connected components. The function which actually
    /// determines a board's connected components is quite expensive!
    pub fn is_cut_cell(&self, cell_idx: u8) -> bool {
        let neighbors = Board::index_to_evenr(cell_idx).neighbors().into_iter();
        let mut saw_live = false;
        let mut crossings = 0;
        for neighbor in neighbors {
            if Board::in_bounds(&neighbor) && !self.is_claimed(Board::evenr_to_index(&neighbor)) {
                saw_live = true;
            } else {
                if saw_live {
                    crossings += 1;
                }
                saw_live = false;
            }
            if crossings >= 2 {
                return true;
            }
        }
        false
    }

    pub fn moves(&self, cell_idx: u8) -> CellSet {
        let cell = Board::index_to_evenr(cell_idx);
        cell.neighbors()
            .into_iter()
            .map(|n| self.legal_moves_in_line(&cell, &n))
            .fold(CellSet::new(), |acc, moves| {
                let mut acc = acc.clone();
                acc.union(&moves);
                acc
            })
    }

    pub fn is_legal_move(&self, p: Player, src: u8, dst: u8) -> bool {
        self.penguins[p.id].contains(src)
            && !self.is_claimed(dst)
            && self.is_clear_path(&Board::index_to_evenr(src), &Board::index_to_evenr(dst))
    }

    /**
     * Return all legal moves in the line connecting src to dst.
     */
    fn legal_moves_in_line(&self, src: &EvenR, dst: &EvenR) -> CellSet {
        if src == dst {
            return CellSet::new();
        }

        line(&src, &dst)
            .skip(1)
            .take_while(|hex| {
                let in_bounds = Board::in_bounds(hex);
                in_bounds && !self.is_claimed(Board::evenr_to_index(hex))
            })
            .map(|cell| Board::evenr_to_index(&cell))
            .collect()
    }

    fn is_clear_path(&self, src: &EvenR, dst: &EvenR) -> bool {
        if self.is_claimed(Board::evenr_to_index(dst)) {
            return false;
        }
        line(src, dst)
            .skip(1)
            .take_while(|hex| hex != dst)
            .map(|hex| Board::evenr_to_index(&hex))
            .all(|idx| !self.is_claimed(idx))
    }

    pub fn evenr_to_index(c: &EvenR) -> u8 {
        let paired_offset = (EVEN_ROW_LEN + ODD_ROW_LEN) as u8 * (c.row / 2) as u8;
        let unpaired_offset = if c.row % 2 == 1 { EVEN_ROW_LEN } else { 0 } as u8;
        paired_offset + unpaired_offset + c.col as u8
    }

    pub fn index_to_evenr(idx: u8) -> EvenR {
        let mut idx = idx as i64;
        let mut row = 0;
        loop {
            let row_len = if row % 2 == 0 {
                EVEN_ROW_LEN
            } else {
                ODD_ROW_LEN
            } as i64;
            if idx < row_len {
                return EvenR {
                    row,
                    col: idx % row_len as i64,
                };
            } else {
                idx -= row_len;
                row += 1;
            }
        }
    }

    pub fn prune(&mut self) -> bool {
        let mut has_done_anything = false;
        let components = self.connected_components();

        // If a penguin is completely inside a connected component,
        // and if that penguin is alone in the connected component,
        // that penguin can no longer interact with the rest of the
        // board.
        for iceberg in &components {
            let penguins_touching_iceberg = self
                .penguins
                .iter()
                .enumerate()
                .flat_map(|(player, penguins)| {
                    penguins
                        .into_iter()
                        .filter(|&p| Board::neighbors(p).any(|neighbor| iceberg.contains(neighbor)))
                        .map(move |p| (Player { id: player }, p))
                })
                .take(2)
                .collect_vec();
            if penguins_touching_iceberg.len() != 1 {
                continue;
            }
            let (player, penguin) = penguins_touching_iceberg[0];
            let can_leave_iceberg = Board::neighbors(penguin)
                .any(|neighbor| !iceberg.contains(neighbor) && !self.is_claimed(neighbor));
            if can_leave_iceberg {
                continue;
            }
            has_done_anything = has_done_anything || self.fill(&player, penguin);
        }

        has_done_anything
    }

    pub fn reap(&mut self) {
        // remove penguins that can no longer move
        self.penguins = self
            .penguins
            .iter()
            .map(|penguins| {
                penguins
                    .into_iter()
                    .filter(|&p| !self.moves(p).is_empty())
                    .collect()
            })
            .collect();
    }

    /// Assuming the given penguin is alone on a connected component,
    /// and if the penguin can touch every tile on that component
    /// exactly once, do so and return true.
    fn fill(&mut self, player: &Player, penguin: u8) -> bool {
        let moves = self.optimal_path(player, penguin);

        if let Some((cells_to_take, _)) = moves {
            let mut penguin = penguin;
            for (_, cell, _) in cells_to_take.into_iter().skip(1) {
                self.move_penguin(*player, penguin, cell).unwrap();
                penguin = cell;
            }
            true
        } else {
            false
        }
    }

    fn optimal_path(
        &self,
        player: &Player,
        penguin: u8,
    ) -> Option<(Vec<(Board, u8, usize)>, usize)> {
        let best_remaining_score_fn = |board: &Board, penguin: u8| -> usize {
            // pick the largest adjacent component
            // and assume we can get the whole thing
            board
                .connected_components()
                .iter()
                .filter(|&iceberg| {
                    Board::neighbors(penguin).any(|adjacency| iceberg.contains(adjacency))
                })
                .map(|iceberg| iceberg.iter().map(|idx| board.num_fish(idx)).sum())
                .max()
                .unwrap_or(0)
        };
        let best_score_from_start = best_remaining_score_fn(self, penguin);

        let soln = pathfinding::dfs::dfs(
            (self.clone(), penguin, 0),
            |&(ref board, src, score)| {
                // Fail early if we can't get a perfect score
                let best_score_from_here = score + best_remaining_score_fn(board, src);
                if best_score_from_here < best_score_from_start {
                    return vec![].into_iter();
                }
                board
                    .moves(src)
                    .into_iter()
                    .filter(|&c| !board.is_claimed(c))
                    .map(move |dst| {
                        let mut new_board = board.clone();
                        new_board.claim_cell(*player, dst).unwrap();
                        let claimed_fish = new_board.num_fish(dst);
                        (new_board, dst, score + claimed_fish)
                    })
                    .collect_vec()
                    .into_iter()
            },
            |&(ref b, penguin, score)| {
                // Stop only when we can't continue on
                if !b.moves(penguin).is_empty() {
                    return false;
                }
                score >= best_score_from_start
            },
        );
        match soln {
            Some(moves) => Some((moves, best_score_from_start)),
            _ => None,
        }
    }

    fn connected_components(&self) -> Vec<CellSet> {
        let claimed_cells = self.all_claimed_cells();
        let num_unclaimed = NUM_CELLS - claimed_cells.len();
        let mut marked = CellSet::new();
        let mut components = vec![];

        while marked.len() < num_unclaimed {
            let mut available = CellSet::full();
            available.exclude(&claimed_cells);
            available.exclude(&marked);

            let idx = available.iter().next().unwrap();
            let new_component = self.component(idx);
            marked.union(&new_component);
            components.push(new_component);
        }
        components
    }

    fn component(&self, start: u8) -> CellSet {
        let mut marked = CellSet::new();
        let mut queue = vec![start];
        while let Some(idx) = queue.pop() {
            marked.insert(idx);
            let mut new_members: CellSet = Board::neighbors(idx).collect();
            new_members.exclude(&self.all_claimed_cells());
            new_members.exclude(&marked);
            for x in new_members.iter() {
                queue.push(x);
            }
        }
        marked
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_board() {
        Board::new([0; 32]);
    }

    #[test]
    fn index_evenr_translation() {
        for idx in 0..NUM_CELLS as u8 {
            assert!(Board::evenr_to_index(&Board::index_to_evenr(idx)) == idx);
        }
    }

    #[test]
    fn claim_cell() {
        let mut b = Board::new([0; 32]);

        let c = 32;
        assert!(!b.is_claimed(c));

        b.claim_cell(Player { id: 1 }, c).unwrap();
        assert!(b.is_claimed(c));
    }

    #[test]
    fn claimed_cell_breaks_path() {
        let mut b = Board::new([0; 32]);

        let c1 = EvenR { col: 1, row: 2 };
        let c2 = EvenR { col: 3, row: 5 };
        assert!(b.is_clear_path(&c1, &c2));

        b.claim_cell(
            Player { id: 1 },
            Board::evenr_to_index(&EvenR { col: 2, row: 3 }),
        )
        .unwrap();
        assert!(!b.is_clear_path(&c1, &c2));
    }

    #[test]
    fn claimed_start() {
        let mut b = Board::new([0; 32]);
        let c1 = EvenR { col: 1, row: 2 };
        let c2 = EvenR { col: 3, row: 5 };
        b.claim_cell(Player { id: 1 }, Board::evenr_to_index(&c1))
            .unwrap();
        assert!(b.is_clear_path(&c1, &c2));
    }

    #[test]
    fn claimed_finish() {
        let mut b = Board::new([0; 32]);
        let c1 = EvenR { col: 1, row: 2 };
        let c2 = EvenR { col: 3, row: 5 };
        b.claim_cell(Player { id: 1 }, Board::evenr_to_index(&c2))
            .unwrap();
        assert!(!b.is_clear_path(&c1, &c2));
    }

    #[test]
    fn nonempty_legal_moves() {
        let b = Board::new([0; 32]);
        let c = EvenR { col: 1, row: 2 };
        let moves = b.moves(Board::evenr_to_index(&c));
        assert_ne!(moves, CellSet::new());
    }

    #[test]
    fn empty_legal_moves() {
        let mut b = Board::new([0; 32]);
        let c = EvenR { col: 1, row: 2 };
        for x in c.neighbors() {
            if Board::in_bounds(&x) {
                b.claim_cell(Player { id: 1 }, Board::evenr_to_index(&x))
                    .unwrap();
            }
        }
        let moves = b.moves(Board::evenr_to_index(&c));
        assert_eq!(moves, CellSet::new());
    }

    #[test]
    fn one_connected_component_at_start() {
        let b = Board::new([0; 32]);
        let components = b.connected_components();
        assert_eq!(components.len(), 1);
        assert_eq!(components[0].len(), NUM_CELLS);
    }

    #[test]
    fn two_connected_components() {
        let mut b = Board::new([0; 32]);

        // carve out upper left corner
        let claimed = vec![1, 7, 8];
        for &x in claimed.iter() {
            b.claim_cell(Player { id: 0 }, x).unwrap();
        }

        let components = b.connected_components();
        assert_eq!(components.len(), 2);
        assert_eq!(components[0].len(), 1);
        assert_eq!(components[1].len(), NUM_CELLS - 1 - claimed.len());
    }

    #[test]
    fn test_cut_cell_in_corner() {
        let mut b = Board::new([0; 32]);

        b.claim_cell(Player { id: 0 }, 8).unwrap();
        assert!(b.is_cut_cell(1));
    }

    #[test]
    fn test_connected_components() {
        fn run_test(seed: [u8; 32]) {
            let mut b = Board::new(seed);

            let mut cells = (0..NUM_CELLS).into_iter().collect_vec();
            let mut rng: StdRng = SeedableRng::from_seed(seed);
            rng.shuffle(&mut cells);

            for c in 0..30 {
                b.claim_cell(Player { id: 0 }, c).unwrap();
            }
            let components = b.connected_components();
            assert_eq!(
                b.claimed_cells().len() + components.iter().flat_map(|x| x.iter()).count(),
                NUM_CELLS,
            );
            let mut all_cells = CellSet::new();
            for component in components {
                for cell in component.iter() {
                    all_cells.insert(cell);
                }
            }
            for cell in b.claimed_cells().iter() {
                all_cells.insert(cell);
            }
            assert_eq!(all_cells.len(), NUM_CELLS);
        }

        for seed in 0..100 {
            run_test([seed; 32]);
        }
    }

    #[test]
    fn one_penguin_prunes_the_whole_board() {
        let mut b = Board::new([0; 32]);
        b.claim_cell(Player { id: 0 }, 0).unwrap();
        b.prune();
        let num_claimed_cells = (0..NUM_CELLS as u8)
            .into_iter()
            .filter(|&c| b.is_claimed(c))
            .count();
        assert_eq!(num_claimed_cells, NUM_CELLS);
    }

    #[test]
    fn two_players_no_pruning() {
        let mut b = Board::new([0; 32]);
        b.claim_cell(Player { id: 0 }, 0).unwrap();
        b.claim_cell(Player { id: 1 }, 1).unwrap();
        b.prune();
        let num_claimed_cells = (0..NUM_CELLS as u8)
            .into_iter()
            .filter(|&c| b.is_claimed(c))
            .count();
        assert_eq!(num_claimed_cells, 2);
    }
}

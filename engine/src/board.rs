use arrayvec::ArrayVec;
use rand::prelude::*;

use crate::cellset::CellSet;
use crate::errors::IllegalMoveError;
use crate::hex::{line, Cube, EvenR};
use crate::{EVEN_ROW_LEN, NUM_CELLS, NUM_ONE_FISH, NUM_THREE_FISH, NUM_TWO_FISH, ODD_ROW_LEN};

type NumFish = usize;
type DFSState = (Board, u8, NumFish);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Player {
    pub id: usize,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Board {
    pub penguins: [CellSet; 2],
    pub fish: [CellSet; 3],
    pub claimed: [CellSet; 2],
}

const RAY_MASKS: [[u64; 6]; 64] = compute_ray_masks();

const fn compute_ray_masks() -> [[u64; 6]; 64] {
    let mut masks = [[0; 6]; 64];
    let mut i = 0;
    while i < 64 {
        let src_idx = i as u8;
        if src_idx >= NUM_CELLS as u8 {
            i += 1;
            continue;
        }
        let src_hex = Board::index_to_evenr(src_idx);
        let src_cube = Cube::from_evenr(&src_hex);

        let mut dir = 0;
        while dir < 6 {
            let mut ray_mask = 0;
            let mut cur_cube = src_cube;
            loop {
                let neighbors = cur_cube.neighbors();
                let next_cube = neighbors[dir];

                let next_hex = EvenR::from_cube(&next_cube);
                if !Board::in_bounds(&next_hex) {
                    break;
                }
                let next_idx = Board::evenr_to_index(&next_hex);
                ray_mask |= 1 << next_idx;
                cur_cube = next_cube;
            }
            masks[i][dir] = ray_mask;
            dir += 1;
        }
        i += 1;
    }
    masks
}

impl Board {
    pub fn new<R: Rng + ?Sized>(rng: &mut R) -> Board {
        let mut cell_to_fish = [1; NUM_CELLS];
        let mut size = NUM_ONE_FISH;

        for _ in 0..NUM_TWO_FISH {
            cell_to_fish[size] = 2;
            size += 1
        }
        for _ in 0..NUM_THREE_FISH {
            cell_to_fish[size] = 3;
            size += 1;
        }

        cell_to_fish.shuffle(rng);

        let mut fish: [CellSet; 3] = [CellSet::new(); 3];
        for i in 0..NUM_CELLS {
            fish[cell_to_fish[i] - 1] = fish[cell_to_fish[i] - 1].insert(i as u8);
        }

        Board {
            fish,
            penguins: [CellSet::new(); 2],
            claimed: [CellSet::new(); 2],
        }
    }

    pub fn num_fish(&self, idx: u8) -> NumFish {
        self.fish
            .iter()
            .enumerate()
            .find(|&(_, cells_with_fish)| cells_with_fish.contains(idx))
            .unwrap()
            .0
            + 1
    }

    pub fn all_claimed_cells(&self) -> CellSet {
        let mut ret = CellSet::new();
        for &cells in self.claimed.iter() {
            ret = ret.union(cells);
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

        self.claimed[p.id] = self.claimed[p.id].insert(idx);
        self.penguins[p.id] = self.penguins[p.id].insert(idx as u8);
        Ok(())
    }

    pub fn get_score(&self, p: Player) -> NumFish {
        (1..=3)
            .map(|num_fish| {
                let claimed_with_fish = self.claimed[p.id].intersect(self.fish[num_fish - 1]);
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
        self.claimed[p.id] = self.claimed[p.id].insert(dst);
        self.penguins[p.id] = self.penguins[p.id].remove(src as u8);
        self.penguins[p.id] = self.penguins[p.id].insert(dst as u8);
        Ok(self)
    }

    fn neighbors(c: u8) -> ArrayVec<u8, 6> {
        Board::index_to_evenr(c)
            .neighbors()
            .iter()
            .filter(|neighbor| Board::in_bounds(neighbor))
            .map(Board::evenr_to_index)
            .collect()
    }

    const fn in_bounds(c: &EvenR) -> bool {
        c.row >= 0 && Board::in_column_bounds(c) && Board::evenr_to_index(c) < NUM_CELLS as u8
    }

    const fn in_column_bounds(c: &EvenR) -> bool {
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
        for &cells in self.claimed.iter() {
            ret = ret.union(cells);
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
        let neighbors = Board::index_to_evenr(cell_idx).neighbors();
        let mut saw_live = false;
        let mut crossings = 0;
        for neighbor in neighbors.iter() {
            if Board::in_bounds(neighbor) && !self.is_claimed(Board::evenr_to_index(neighbor)) {
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
        let mut moves = 0;
        let occupied = self.claimed[0].data | self.claimed[1].data;

        // For each of the 6 directions
        for dir in 0..6 {
            let ray = RAY_MASKS[cell_idx as usize][dir];
            
            // If we intersect with occupied cells:
            let blockers = ray & occupied;
            if blockers != 0 {
                // There is a blocker.
                // If the direction is increasing index (forward), the first blocker is the one with the smallest index (trailing_zeros).
                // If the direction is decreasing index (backward), the first blocker is the one with the largest index (leading_zeros).
                
                // We need to know which directions are "positive" and "negative" in terms of bit index.
                // 0: East (+1) -> Positive
                // 1: NorthEast (depends on row parity, but generally -row_len) -> Negative
                // 2: NorthWest -> Negative
                // 3: West (-1) -> Negative
                // 4: SouthWest -> Positive
                // 5: SouthEast -> Positive
                
                if dir == 0 || dir == 4 || dir == 5 {
                    // Positive direction: we want everything strictly less than the smallest blocker
                    let first_blocker_idx = blockers.trailing_zeros();
                    // Create a mask of all bits less than first_blocker_idx
                    let mask = (1u64 << first_blocker_idx) - 1;
                    moves |= ray & mask;
                } else {
                    // Negative direction: we want everything strictly greater than the largest blocker
                    let first_blocker_idx = 63 - blockers.leading_zeros();
                    // Create a mask of all bits greater than first_blocker_idx
                    // (1 << (idx + 1)) - 1 gives 0..idx. We want the opposite.
                    // !((1 << (idx + 1)) - 1)
                    // Or simpler: !((1 << (first_blocker_idx + 1)).wrapping_sub(1))
                    // Be careful with overflow if idx is 63.
                    
                    let mask = if first_blocker_idx == 63 {
                        0
                    } else {
                        !((1u64 << (first_blocker_idx + 1)) - 1)
                    };
                    moves |= ray & mask;
                }
            } else {
                // No blockers, all potential moves in this ray are valid
                moves |= ray;
            }
        }
        
        CellSet { data: moves }
    }

    pub fn is_legal_move(&self, p: Player, src: u8, dst: u8) -> bool {
        self.penguins[p.id].contains(src)
            && !self.is_claimed(dst)
            && self.is_clear_path(&Board::index_to_evenr(src), &Board::index_to_evenr(dst))
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

    pub const fn evenr_to_index(c: &EvenR) -> u8 {
        let paired_offset = (EVEN_ROW_LEN + ODD_ROW_LEN) as u8 * (c.row / 2) as u8;
        let unpaired_offset = if c.row % 2 == 1 { EVEN_ROW_LEN } else { 0 } as u8;
        paired_offset + unpaired_offset + c.col as u8
    }

    pub const fn index_to_evenr(idx: u8) -> EvenR {
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

    /// Iff a penguin is alone on the board and can catch every fish on its
    /// "iceberg", claim all the fish on the iceberg
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
                        .filter(|&p| {
                            Board::neighbors(p)
                                .into_iter()
                                .any(|neighbor| iceberg.contains(neighbor))
                        })
                        .map(move |p| (Player { id: player }, p))
                })
                .take(2)
                .collect::<ArrayVec<_, 2>>();
            if penguins_touching_iceberg.len() != 1 {
                continue;
            }
            let (player, penguin) = penguins_touching_iceberg[0];
            let can_leave_iceberg = Board::neighbors(penguin)
                .into_iter()
                .any(|neighbor| !iceberg.contains(neighbor) && !self.is_claimed(neighbor));
            if can_leave_iceberg {
                continue;
            }
            has_done_anything = has_done_anything || self.fill(player, penguin);
        }

        has_done_anything
    }

    /// remove penguins that can no longer move
    pub fn reap(&mut self) {
        let mut new_penguins: [CellSet; 2] = [CellSet::new(); 2];
        for (i, penguins) in self.penguins.iter().enumerate() {
            new_penguins[i] = penguins
                .into_iter()
                .filter(|&p| Board::neighbors(p).into_iter().any(|n| !self.is_claimed(n)))
                .collect();
        }
        self.penguins = new_penguins
    }

    /// Assuming the given penguin is alone on a connected component,
    /// and if the penguin can touch every tile on that component
    /// exactly once, do so and return true.
    fn fill(&mut self, player: Player, penguin: u8) -> bool {
        let moves = self.optimal_path(player, penguin);

        if let Some((cells_to_take, _)) = moves {
            let mut penguin = penguin;
            for (_, cell, _) in cells_to_take.into_iter().skip(1) {
                self.move_penguin(player, penguin, cell).unwrap();
                penguin = cell;
            }
            true
        } else {
            false
        }
    }

    fn optimal_path(&self, player: Player, penguin: u8) -> Option<(Vec<DFSState>, NumFish)> {
        let best_score_from_start = self.best_score_for_penguin(penguin);
        Self::dfs_this_iceberg(player, best_score_from_start, self.clone(), penguin, 0)
            .map(|moves| (moves, best_score_from_start))
    }

    /// If there is a way for the penguin located at `src` to accrue
    /// `best_score_from_start` points without interacting with any
    /// other penguins, return one of the paths by which this may be
    /// accomplished.
    ///
    /// `board` represents the current board state, `src` represents
    /// the current location of the penguin, and `score` represents
    /// the number of points the penguin has accumulated.
    ///
    /// We succeed when `score` equals `best_score_from_start` and
    /// fail if we can no longer accumulate `best_score_from_start`
    /// points from this position.
    fn dfs_this_iceberg(
        player: Player,
        best_score_from_start: NumFish,
        board: Board,
        src: u8,
        score: NumFish,
    ) -> Option<Vec<DFSState>> {
        // Succeed early if we attained a perfect score by
        // touching every cell on our connected component (aka
        // "iceberg")
        assert!(score <= best_score_from_start);
        if score == best_score_from_start {
            return Some(vec![(board, src, score)]);
        }

        // Fail early if we can't get a perfect score
        let best_score_from_here = score + board.best_score_for_penguin(src);
        if best_score_from_here < best_score_from_start {
            return None;
        }

        let new_states = board
            .moves(src)
            .into_iter()
            .filter(|&c| !board.is_claimed(c))
            .map(|dst| {
                let mut new_board = board.clone();
                new_board.claim_cell(player, dst).unwrap();
                let claimed_fish = new_board.num_fish(dst);
                (new_board, dst, score + claimed_fish)
            });
        for (new_board, new_src, new_score) in new_states {
            if let Some(moves) =
                Self::dfs_this_iceberg(player, best_score_from_start, new_board, new_src, new_score)
            {
                let mut moves_including_start = vec![(board, src, score)];
                moves_including_start.extend(moves.into_iter());
                return Some(moves_including_start);
            }
        }
        None
    }

    /// Return the number of points a penguin at the given location could get,
    /// assuming no other penguins ever move.
    fn best_score_for_penguin(&self, penguin: u8) -> NumFish {
        // pick the largest adjacent component
        // and assume we can get the whole thing
        self.connected_components()
            .iter()
            .filter(|&iceberg| {
                Board::neighbors(penguin)
                    .into_iter()
                    .any(|adjacency| iceberg.contains(adjacency))
            })
            .map(|iceberg| iceberg.iter().map(|idx| self.num_fish(idx)).sum())
            .max()
            .unwrap_or(0)
    }

    /// Return a vector of all connected components on the board, also called "icebergs".
    fn connected_components(&self) -> Vec<CellSet> {
        let claimed_cells = self.all_claimed_cells();
        let num_unclaimed = NUM_CELLS - claimed_cells.len();
        let mut marked = CellSet::new();
        let mut components = vec![];

        while marked.len() < num_unclaimed {
            let available = CellSet::full().exclude(claimed_cells).exclude(marked);

            let idx = available.iter().next().unwrap();
            let new_component = self.component(idx);
            marked = marked.union(new_component);
            components.push(new_component);
        }
        components
    }

    fn component(&self, start: u8) -> CellSet {
        let mut marked = CellSet::new();
        let mut queue = vec![start];
        while let Some(idx) = queue.pop() {
            marked = marked.insert(idx);
            let new_members: CellSet = Board::neighbors(idx)
                .into_iter()
                .collect::<CellSet>()
                .exclude(self.all_claimed_cells())
                .exclude(marked);
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
        Board::new::<StdRng>(&mut SeedableRng::seed_from_u64(0));
    }

    #[test]
    fn index_evenr_translation() {
        for idx in 0..NUM_CELLS as u8 {
            assert!(Board::evenr_to_index(&Board::index_to_evenr(idx)) == idx);
        }
    }

    #[test]
    fn claim_cell() {
        let mut b = Board::new::<StdRng>(&mut SeedableRng::seed_from_u64(0));

        let c = 32;
        assert!(!b.is_claimed(c));

        b.claim_cell(Player { id: 1 }, c).unwrap();
        assert!(b.is_claimed(c));
    }

    #[test]
    fn claimed_cell_breaks_path() {
        let mut b = Board::new::<StdRng>(&mut SeedableRng::seed_from_u64(0));

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
        let mut b = Board::new::<StdRng>(&mut SeedableRng::seed_from_u64(0));
        let c1 = EvenR { col: 1, row: 2 };
        let c2 = EvenR { col: 3, row: 5 };
        b.claim_cell(Player { id: 1 }, Board::evenr_to_index(&c1))
            .unwrap();
        assert!(b.is_clear_path(&c1, &c2));
    }

    #[test]
    fn claimed_finish() {
        let mut b = Board::new::<StdRng>(&mut SeedableRng::seed_from_u64(0));
        let c1 = EvenR { col: 1, row: 2 };
        let c2 = EvenR { col: 3, row: 5 };
        b.claim_cell(Player { id: 1 }, Board::evenr_to_index(&c2))
            .unwrap();
        assert!(!b.is_clear_path(&c1, &c2));
    }

    #[test]
    fn nonempty_legal_moves() {
        let b = Board::new::<StdRng>(&mut SeedableRng::seed_from_u64(0));
        let c = EvenR { col: 1, row: 2 };
        let moves = b.moves(Board::evenr_to_index(&c));
        assert_ne!(moves, CellSet::new());
    }

    #[test]
    fn empty_legal_moves() {
        let mut b = Board::new::<StdRng>(&mut SeedableRng::seed_from_u64(0));
        let c = EvenR { col: 1, row: 2 };
        for x in c.neighbors().iter() {
            if Board::in_bounds(x) {
                b.claim_cell(Player { id: 1 }, Board::evenr_to_index(x))
                    .unwrap();
            }
        }
        let moves = b.moves(Board::evenr_to_index(&c));
        assert_eq!(moves, CellSet::new());
    }

    #[test]
    fn one_connected_component_at_start() {
        let b = Board::new::<StdRng>(&mut SeedableRng::seed_from_u64(0));
        let components = b.connected_components();
        assert_eq!(components.len(), 1);
        assert_eq!(components[0].len(), NUM_CELLS);
    }

    #[test]
    fn two_connected_components() {
        let mut b = Board::new::<StdRng>(&mut SeedableRng::seed_from_u64(0));

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
        let mut b = Board::new::<StdRng>(&mut SeedableRng::seed_from_u64(0));

        b.claim_cell(Player { id: 0 }, 8).unwrap();
        assert!(b.is_cut_cell(1));
    }

    #[test]
    fn test_connected_components() {
        fn run_test(seed: [u8; 32]) {
            let mut b = Board::new::<StdRng>(&mut SeedableRng::seed_from_u64(0));

            let mut cells: Vec<_> = (0..NUM_CELLS).collect();
            let mut rng: StdRng = SeedableRng::from_seed(seed);
            cells.shuffle(&mut rng);

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
                    all_cells = all_cells.insert(cell);
                }
            }
            for cell in b.claimed_cells().iter() {
                all_cells = all_cells.insert(cell);
            }
            assert_eq!(all_cells.len(), NUM_CELLS);
        }

        for seed in 0..100 {
            run_test([seed; 32]);
        }
    }

    #[test]
    fn one_penguin_prunes_the_whole_board() {
        let mut b = Board::new::<StdRng>(&mut SeedableRng::seed_from_u64(0));
        b.claim_cell(Player { id: 0 }, 0).unwrap();
        b.prune();
        let num_claimed_cells = (0..NUM_CELLS as u8).filter(|&c| b.is_claimed(c)).count();
        assert_eq!(num_claimed_cells, NUM_CELLS);
    }

    #[test]
    fn two_players_no_pruning() {
        let mut b = Board::new::<StdRng>(&mut SeedableRng::seed_from_u64(0));
        b.claim_cell(Player { id: 0 }, 0).unwrap();
        b.claim_cell(Player { id: 1 }, 1).unwrap();
        b.prune();
        let num_claimed_cells = (0..NUM_CELLS as u8).filter(|&c| b.is_claimed(c)).count();
        assert_eq!(num_claimed_cells, 2);
    }

    #[cfg(test)]
    mod quickcheck_tests {
        use super::*;
        use quickcheck::TestResult;
        use quickcheck_macros::quickcheck;

        fn ref_legal_moves_in_line(board: &Board, src: &EvenR, dst: &EvenR) -> CellSet {
            if src == dst {
                return CellSet::new();
            }

            let mut ret = CellSet::new();
            for hex in line(src, dst) {
                if &hex == src {
                    continue;
                }
                if !Board::in_bounds(&hex) {
                    break;
                }
                let idx = Board::evenr_to_index(&hex);
                if board.is_claimed(idx) {
                    break;
                }
                ret = ret.insert(idx);
            }
            ret
        }

        fn ref_moves(board: &Board, cell_idx: u8) -> CellSet {
            let cell = Board::index_to_evenr(cell_idx);
            cell.neighbors()
                .iter()
                .map(|n| ref_legal_moves_in_line(board, &cell, n))
                .fold(CellSet::new(), |acc, moves| acc.union(moves))
        }

        #[quickcheck]
        fn prop_moves_equivalence(
            src_idx: u8,
            claimed_mask_0: u64,
            claimed_mask_1: u64,
        ) -> TestResult {
            if src_idx >= NUM_CELLS as u8 {
                return TestResult::discard();
            }

            let mut board = Board::new::<StdRng>(&mut SeedableRng::seed_from_u64(0));
            // Overwrite claimed sets with random data
            board.claimed[0] = CellSet { data: claimed_mask_0 };
            board.claimed[1] = CellSet { data: claimed_mask_1 };

            // Ensure src is not claimed (moves() might assume it? No, but usually we move from our own penguin)
            // But moves() signature is just (cell_idx).
            // If src is claimed, moves() should still return valid moves from that cell (as if a penguin was there).
            // The logic doesn't check if src is claimed.

            let optimized = board.moves(src_idx);
            let reference = ref_moves(&board, src_idx);

            TestResult::from_bool(optimized == reference)
        }
    }
}

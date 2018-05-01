extern crate bit_set;
extern crate itertools;
extern crate pathfinding;

use rand::{Rng, SeedableRng, StdRng};

use self::itertools::Itertools;
use self::bit_set::BitSet;

use errors::IllegalMoveError;
use hex::{EvenR, line};

const EVEN_ROW_LEN: usize = 7;
const ODD_ROW_LEN: usize = 8;
pub const NUM_CELLS: usize = (EVEN_ROW_LEN * (NUM_ROWS / 2) +
                          ODD_ROW_LEN * (NUM_ROWS / 2));
const NUM_ROWS: usize = 8;
const NUM_ONE_FISH: usize = 30;
const NUM_TWO_FISH: usize = 20;
const NUM_THREE_FISH: usize = 10;
pub const NUM_FISH: usize = NUM_ONE_FISH + NUM_TWO_FISH + NUM_THREE_FISH;

type Fish = usize;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Player {
    pub id: usize,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Cell {
    pub fish: Fish,
    pub claimed: Option<Player>,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Board {
    pub penguins: Vec<BitSet<u32>>,
    pub cells: Vec<Cell>,
}

impl Board {

    pub fn new(seed: &[usize]) -> Board {
        let mut fish: [Fish; NUM_FISH] = [1; NUM_FISH];
        let mut size: usize = NUM_ONE_FISH;

        for _ in 0..NUM_TWO_FISH {
            fish[size] = 2;
            size += 1
        }
        for _ in 0..NUM_THREE_FISH {
            fish[size] = 3;
            size += 1;
        }

        {
            let mut rng: StdRng = SeedableRng::from_seed(seed);
            rng.shuffle(&mut fish);
        }

        let mut cells: [Cell; NUM_CELLS] = [
            Cell{fish: 1, claimed: None};
            NUM_CELLS
        ];
        for (i, cell) in cells.iter_mut().enumerate() {
            cell.fish = fish[i];
        }

        Board {
            cells: cells.to_vec(),
            penguins: vec![BitSet::new(), BitSet::new()],
        }
    }

    pub fn claim_cell(
        &mut self,
        p: Player,
        idx: usize,
    ) -> Result<(), IllegalMoveError> {
        if let Some(other_p) = self.cells[idx].claimed {
            return Err(
                IllegalMoveError::new(
                    p,
                    format!(
                        "Cell at {} already claimed by player {}",
                        idx,
                        other_p.id
                    )
                )
            );
        }

        self.cells[idx].claimed = Some(p);
        self.penguins[p.id].insert(idx);
        Ok(())
    }

    pub fn move_penguin(
        &mut self,
        p: Player,
        src: usize,
        dst: usize,
    ) -> Result<&Self, IllegalMoveError> {
        if !self.is_legal_move(p, src, dst) {
            return Err(
                IllegalMoveError::new(
                    p,
                    format!(
                        "Player {:?} cannot move penguin from {:?} to {:?}",
                        p,
                        src,
                        dst
                    )
                )
            );
        }
        self.cells[dst].claimed = Some(p);
        self.penguins[p.id].remove(src);
        self.penguins[p.id].insert(dst);
        Ok(self)
    }

    fn get<'a>(&'a self, c: &EvenR) -> &'a Cell {
        &self.cells[Board::evenr_to_index(c)]
    }

    fn neighbors(c: usize) -> Vec<usize> {
        Board::index_to_evenr(c)
            .neighbors()
            .into_iter()
            .filter(|neighbor| Board::in_bounds(neighbor))
            .map(|evenr| Board::evenr_to_index(&evenr))
            .collect()
    }

    fn in_bounds(c: &EvenR) -> bool {
        c.row >= 0 &&
            Board::in_column_bounds(c) &&
            Board::evenr_to_index(c) < NUM_CELLS
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

    pub fn claimed_cells(&self) -> Vec<usize> {
        self.cells.iter().enumerate()
            .filter(|&(_, c)| c.claimed.is_some())
            .map(|(idx, _)| idx)
            .collect()
    }

    pub fn moves(&self, cell_idx: usize) -> Vec<usize> {
        let cell = Board::index_to_evenr(cell_idx);
        cell.neighbors().iter()
            .flat_map(|&n| self.legal_moves_in_line(&cell, &n).into_iter())
            .map(|cell| Board::evenr_to_index(&cell))
            .into_iter()
            .collect()
    }

    pub fn is_legal_move(
        &self,
        p: Player,
        src: usize,
        dst: usize,
    ) -> bool {
        (self.cells[src].claimed == Some(p) &&
         self.cells[dst].claimed == None &&
         self.is_clear_path(
             &Board::index_to_evenr(src),
             &Board::index_to_evenr(dst),
         ))
    }

    /**
     * Return all legal moves in the line connecting src to dst.
     */
    fn legal_moves_in_line(
        &self,
        src: &EvenR,
        dst: &EvenR,
    ) -> Vec<EvenR> {
        if src == dst {
            return vec![];
        }

        line(&src, &dst)
            .skip(1)
            .take_while(|x| {
                let in_bounds = Board::in_bounds(x);
                in_bounds && self.get(x).claimed == None
            })
            .collect()
    }

    fn is_clear_path(&self, src: &EvenR, dst: &EvenR) -> bool {
        line(src, dst).skip(1)
            .take_while(|hex| hex != dst)
            .all(|hex| self.get(&hex).claimed == None) &&
            self.get(&dst).claimed == None

    }

    pub fn get_score(&self, player: &Player) -> usize {
        self.cells.iter()
            .filter(|&c| c.claimed == Some(*player))
            .map(|c| c.fish)
            .sum()
    }

    pub fn evenr_to_index(c: &EvenR) -> usize {
        let paired_offset = (EVEN_ROW_LEN + ODD_ROW_LEN) * (c.row / 2) as usize;
        let unpaired_offset = if c.row % 2 == 1 {
            EVEN_ROW_LEN
        } else {
            0
        } as usize;
        paired_offset + unpaired_offset + c.col as usize
    }

    pub fn index_to_evenr(idx: usize) -> EvenR {
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
            let penguins_touching_iceberg = self.penguins
                .iter()
                .enumerate()
                .flat_map(|(player, penguins)| {
                    penguins.into_iter()
                        .filter(|&p| Board::neighbors(p).iter().any(
                            |&neighbor| {
                                iceberg.contains(neighbor)
                            }
                        ))
                        .map(move |p| (Player{id:player}, p))
                })
                .take(2)
                .collect_vec();
            if penguins_touching_iceberg.len() != 1 {
                continue;
            }
            let (player, penguin) = penguins_touching_iceberg[0];
            let can_leave_iceberg = Board::neighbors(penguin).iter()
                .any(|&neighbor| {
                    !iceberg.contains(neighbor) &&
                        self.cells[neighbor].claimed.is_none()
                });
            if can_leave_iceberg {
                continue;
            }
            has_done_anything = has_done_anything || self.fill(&player, penguin);
        }

        has_done_anything
    }

    pub fn reap(&mut self) {
        // remove penguins that can no longer move
        self.penguins = self.penguins.iter().map(
            |penguins| penguins.into_iter().filter(
                |&p| !self.moves(p).is_empty()
            ).collect()
        ).collect();
    }

    /// Assuming the given penguin is alone on a connected component,
    /// and if the penguin can touch every tile on that component
    /// exactly once, do so and return true.
    fn fill(
        &mut self,
        player: &Player,
        penguin: usize,
    ) -> bool {
        let moves = self.optimal_path(player, penguin);

        if let Some((cells_to_take, _)) = moves {
            let mut penguin = penguin;
            for (_, cell) in cells_to_take.into_iter().skip(1) {
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
        penguin: usize,
    ) -> Option<(Vec<(Board, usize)>, usize)> {
        let score = |board: &Board| -> usize {
            board.cells.iter()
                .filter(|c| c.claimed == Some(*player))
                .map(|c| c.fish)
                .sum()
        };
        let starting_score: usize = score(self);

        let best_remaining_score_fn = |board: &Board, penguin: usize| -> usize {
            // pick the largest adjacent component
            // and assume we can get the whole thing
            board.connected_components()
                .iter()
                .filter(
                    |&iceberg| Board::neighbors(penguin).iter()
                        .any(|&adjacency| iceberg.contains(adjacency))
                )
                .map(|iceberg| iceberg.iter()
                     .map(|idx| board.cells[idx].fish)
                     .sum()
                )
                .max()
                .unwrap_or(0)
        };
        let best_score_from_start = best_remaining_score_fn(self, penguin) +
            starting_score;

        let soln = pathfinding::dfs::dfs(
            (self.clone(), penguin),
            |&(ref board, src)| {
                // Fail early if we can't get a perfect score
                let best_score_from_here = score(board) + best_remaining_score_fn(board, src);
                if best_score_from_here < best_score_from_start {
                    return vec![].into_iter();
                }
                board.moves(src).into_iter()
                    .filter(|&c| board.cells[c].claimed.is_none())
                    .map(move |dst| {
                        let mut new_board = board.clone();
                        new_board.claim_cell(*player, dst).unwrap();
                        (new_board, dst)
                    }).collect_vec().into_iter()
            },
            |&(ref b, penguin)| {
                // Stop only when we can't continue on
                if !b.moves(penguin).is_empty() {
                    return false;
                }
                let s = score(b);
                s >= best_score_from_start
            },
        );
        match soln {
            Some(moves) => Some((moves, best_score_from_start)),
            _ => None,
        }
    }

    fn connected_components(&self) -> Vec<BitSet<u32>> {
        let num_unclaimed = (0..NUM_CELLS)
            .filter(|&idx| self.cells[idx].claimed.is_none())
            .count();
        let mut marked = BitSet::new();
        let mut components = vec![];

        while marked.len() < num_unclaimed {
            let idx = (0..NUM_CELLS)
                .filter(
                    |&idx| !marked.contains(idx) &&
                        self.cells[idx].claimed.is_none()
                )
                .nth(0).unwrap();
            let new_component = self.component(idx);
            for x in new_component.iter() {
                marked.insert(x);
            }
            components.push(new_component);
        }
        components
    }

    fn component(&self, start: usize) -> BitSet<u32> {
        let mut marked = BitSet::new();
        let mut queue = vec![start];
        while let Some(idx) = queue.pop() {
            marked.insert(idx);
            let new_members = Board::neighbors(idx)
                .into_iter()
                .filter(
                    |&idx| self.cells[idx].claimed.is_none() &&
                        !marked.contains(idx)
                );
            for x in new_members {
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
        Board::new(&[0]);
    }

    #[test]
    fn index_evenr_translation() {
        for idx in 0..NUM_CELLS {
            assert!(
                Board::evenr_to_index(
                    &Board::index_to_evenr(idx)
                ) == idx
            );
        }
    }

    #[test]
    fn claimed_cell_breaks_path() {
        let mut b = Board::new(&[0]);

        let c1 = EvenR{col: 1, row: 2};
        let c2 = EvenR{col: 3, row: 5};
        assert!(b.is_clear_path(&c1, &c2));

        b.claim_cell(
            Player{id:1},
            Board::evenr_to_index(&EvenR{col: 2, row:3}
        )).unwrap();
        assert!(!b.is_clear_path(&c1, &c2));
    }

    #[test]
    fn claimed_start() {
        let mut b = Board::new(&[0]);
        let c1 = EvenR{col: 1, row: 2};
        let c2 = EvenR{col: 3, row: 5};
        b.claim_cell(Player{id:1}, Board::evenr_to_index(&c1)).unwrap();
        assert!(b.is_clear_path(&c1, &c2));
    }

    #[test]
    fn claimed_finish() {
        let mut b = Board::new(&[0]);
        let c1 = EvenR{col: 1, row: 2};
        let c2 = EvenR{col: 3, row: 5};
        b.claim_cell(Player{id:1}, Board::evenr_to_index(&c2)).unwrap();
        assert!(!b.is_clear_path(&c1, &c2));
    }

    #[test]
    fn nonempty_legal_moves() {
        let b = Board::new(&[0]);
        let c = EvenR{col:1, row:2};
        let moves = b.moves(Board::evenr_to_index(&c));
        assert_ne!(moves, vec![]);
    }

    #[test]
    fn empty_legal_moves() {
        let mut b = Board::new(&[0]);
        let c = EvenR{col:1, row:2};
        for x in c.neighbors() {
            if Board::in_bounds(&x) {
                b.claim_cell(Player{id:1}, Board::evenr_to_index(&x))
                    .unwrap();
            }
        }
        let moves = b.moves(Board::evenr_to_index(&c));
        assert_eq!(moves, vec![]);
    }

    #[test]
    fn one_connected_component_at_start() {
        let b = Board::new(&[0]);
        let components = b.connected_components();
        assert_eq!(components.len(), 1);
        assert_eq!(components[0].len(), NUM_CELLS);
    }

    #[test]
    fn two_connected_components() {
        let mut b = Board::new(&[0]);

        // carve out upper left corner
        let claimed = vec![1, 7, 8];
        for &x in claimed.iter() {
            b.claim_cell(Player{id:0}, x).unwrap();
        }

        let components = b.connected_components();
        assert_eq!(components.len(), 2);
        assert_eq!(components[0].len(), 1);
        assert_eq!(components[1].len(), NUM_CELLS - 1 - claimed.len());
    }

    #[test]
    fn test_connected_components() {
        fn run_test(seed: &[usize]) {
            let mut b = Board::new(seed);

            let mut cells = (0..NUM_CELLS).into_iter().collect_vec();
            let mut rng: StdRng = SeedableRng::from_seed(seed);
            rng.shuffle(&mut cells);

            for c in cells.into_iter().take(30) {
                b.claim_cell(Player{id:0}, c).unwrap();
            }
            let components = b.connected_components();
            assert_eq!(
                b.claimed_cells().len() +
                    components.iter().flat_map(|x| x).count(),
                NUM_CELLS,
            );
            let mut all_cells = BitSet::new();
            for component in components {
                for cell in component.iter() {
                    all_cells.insert(cell);
                }
            }
            for cell in b.claimed_cells() {
                all_cells.insert(cell);
            }
            assert_eq!(all_cells.len(), NUM_CELLS);
        }

        for seed in 0..100 {
            run_test(&[seed]);
        }
    }

    #[test]
    fn one_penguin_prunes_the_whole_board() {
        let mut b = Board::new(&[0]);
        b.claim_cell(Player{id:0}, 0).unwrap();
        b.prune();
        let num_claimed_cells = b.cells.iter()
            .filter(|c| c.claimed.is_some())
            .count();
        assert_eq!(num_claimed_cells, NUM_CELLS);
    }

    #[test]
    fn two_players_no_pruning() {
        let mut b = Board::new(&[0]);
        b.claim_cell(Player{id:0}, 0).unwrap();
        b.claim_cell(Player{id:1}, 1).unwrap();
        b.prune();
        let num_claimed_cells = b.cells.iter()
            .filter(|c| c.claimed.is_some())
            .count();
        assert_eq!(num_claimed_cells, 2);
    }
}

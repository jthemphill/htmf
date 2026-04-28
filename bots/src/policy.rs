use htmf::NUM_CELLS;

use crate::mctsbot::Move;

pub const NUM_DIRECTIONS: usize = 6;
pub const MAX_DISTANCE: usize = 7;
pub const POLICY_VERSION: u8 = 2;
pub const MOVEMENT_POLICY_SIZE: usize = NUM_CELLS * NUM_DIRECTIONS * MAX_DISTANCE;

/// Convert a move from (src, dst) to (direction, distance).
///
/// Direction is 0-5 using the same cube-axis convention everywhere policy
/// targets and priors are encoded.
pub fn move_to_direction_distance(src: u8, dst: u8) -> Option<(usize, usize)> {
    let src_hex = htmf::board::Board::index_to_evenr(src);
    let dst_hex = htmf::board::Board::index_to_evenr(dst);
    let src_cube = htmf::hex::Cube::from_evenr(&src_hex);
    let dst_cube = htmf::hex::Cube::from_evenr(&dst_hex);

    let dx = dst_cube.x - src_cube.x;
    let dy = dst_cube.y - src_cube.y;
    let dz = dst_cube.z - src_cube.z;

    let direction = if dz == 0 {
        if dx > 0 {
            0
        } else {
            3
        }
    } else if dy == 0 {
        if dx > 0 {
            1
        } else {
            4
        }
    } else if dx == 0 {
        if dy > 0 {
            2
        } else {
            5
        }
    } else {
        return None;
    };

    let distance = dx.abs().max(dy.abs()).max(dz.abs()) as usize;
    if distance == 0 || distance > MAX_DISTANCE {
        return None;
    }

    Some((direction, distance))
}

/// Convert a game move to the model policy output index.
///
/// Drafting uses one logit per board cell. Movement uses the absolute
/// `src_cell * 42 + direction * 7 + (distance - 1)` encoding.
pub fn move_to_policy_index(m: &Move, is_drafting: bool) -> usize {
    match m {
        Move::Place(dst) => {
            debug_assert!(is_drafting);
            *dst as usize
        }
        Move::Move((src, dst)) => {
            debug_assert!(!is_drafting);
            if let Some((direction, distance)) = move_to_direction_distance(*src, *dst) {
                *src as usize * (NUM_DIRECTIONS * MAX_DISTANCE)
                    + direction * MAX_DISTANCE
                    + (distance - 1)
            } else {
                0
            }
        }
    }
}

pub fn policy_size(is_drafting: bool) -> usize {
    if is_drafting {
        NUM_CELLS
    } else {
        MOVEMENT_POLICY_SIZE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placement_policy_index_is_cell_index() {
        assert_eq!(move_to_policy_index(&Move::Place(17), true), 17);
    }

    #[test]
    fn movement_policy_index_uses_source_cell() {
        let (direction, distance) = move_to_direction_distance(11, 12).unwrap();
        let expected = 11 * (NUM_DIRECTIONS * MAX_DISTANCE)
            + direction * MAX_DISTANCE
            + (distance - 1);

        assert_eq!(
            move_to_policy_index(&Move::Move((11, 12)), false),
            expected
        );
    }

    #[test]
    fn movement_policy_index_stays_in_range_for_all_board_moves() {
        let mut rng = rand::rng();
        let board = htmf::board::Board::new(&mut rng);

        for src in 0..NUM_CELLS as u8 {
            for dst in board.moves(src) {
                let idx = move_to_policy_index(&Move::Move((src, dst)), false);
                assert!(idx < MOVEMENT_POLICY_SIZE);
            }
        }
    }
}

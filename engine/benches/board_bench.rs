use criterion::{criterion_group, criterion_main, Criterion};
use htmf::board::Board;
use rand::prelude::*;
use std::hint::black_box;

fn benchmark_board_moves(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(0);
    let board = Board::new(&mut rng);

    // Find a cell with moves
    let mut src_idx = 0;
    let mut spaces: Vec<u8> = (0..60).collect();
    spaces.shuffle(&mut rng);
    for i in spaces {
        if !board.moves(i).is_empty() {
            src_idx = i;
            break;
        }
    }

    c.bench_function("board_moves", |b| b.iter(|| {
        black_box(board.moves(black_box(src_idx)))
    }));
}

criterion_group!(benches, benchmark_board_moves);
criterion_main!(benches);

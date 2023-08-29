#[macro_use]
extern crate criterion;

extern crate htmf;
extern crate htmf_bots;

use criterion::Criterion;
use rand::prelude::*;

use htmf::board::*;
use htmf::game::*;
use htmf_bots::*;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("minimax_full_game", |b| {
        b.iter(|| {
            let mut game = GameState::new_two_player::<StdRng>(&mut SeedableRng::seed_from_u64(0));
            let mut bots = vec![
                MinimaxBot::new_with_ply(game.clone(), Player { id: 0 }, 1),
                MinimaxBot::new_with_ply(game.clone(), Player { id: 1 }, 0),
            ];
            while let Some(player) = game.active_player() {
                {
                    let bot = &mut bots[player.id];
                    let action = bot.take_action();
                    game.apply_action(&action).unwrap();
                }
                for bot in &mut bots {
                    bot.update(&game);
                }
            }
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

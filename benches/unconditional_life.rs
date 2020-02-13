use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tsumego_solver::go::GoGame;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("mixture", |b| {
        let game = GoGame::from_sgf(include_str!("../src/test_sgfs/life_and_death/mixture.sgf"));

        b.iter(|| game.get_board().unconditionally_alive_blocks())
    });

    c.bench_function("all_alive1", |b| {
        let game = GoGame::from_sgf(include_str!(
            "../src/test_sgfs/life_and_death/all_alive1.sgf"
        ));

        b.iter(|| game.get_board().unconditionally_alive_blocks())
    });

    c.bench_function("all_dead1", |b| {
        let game = GoGame::from_sgf(include_str!(
            "../src/test_sgfs/life_and_death/all_dead1.sgf"
        ));

        b.iter(|| game.get_board().unconditionally_alive_blocks())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

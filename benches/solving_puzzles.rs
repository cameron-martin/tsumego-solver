use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tsumego_solver::go::{GoGame, GoPlayer};
use tsumego_solver::puzzle::Puzzle;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("true simple 1", |b| {
        let tsumego = black_box(GoGame::from_sgf(include_str!(
            "../src/test_sgfs/puzzles/true_simple1.sgf"
        )));

        b.iter(|| {
            let mut puzzle = Puzzle::new(tsumego.clone(), GoPlayer::White);

            puzzle.solve();
        })
    });

    let mut large_group = c.benchmark_group("Large size");
    large_group.sample_size(10);

    large_group.bench_function("true simple 3", |b| {
        let tsumego = black_box(GoGame::from_sgf(include_str!(
            "../src/test_sgfs/puzzles/true_simple3.sgf"
        )));

        b.iter(|| {
            let mut puzzle = Puzzle::new(tsumego.clone(), GoPlayer::Black);

            puzzle.solve();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use tsumego_solver::puzzle::Puzzle;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("true simple 1", |b| {
        b.iter_batched(
            || Puzzle::from_sgf(include_str!("../src/test_sgfs/puzzles/true_simple1.sgf")),
            |mut puzzle| puzzle.solve(),
            BatchSize::SmallInput,
        )
    });

    c.bench_function("true simple 2", |b| {
        b.iter_batched(
            || Puzzle::from_sgf(include_str!("../src/test_sgfs/puzzles/true_simple2.sgf")),
            |mut puzzle| puzzle.solve(),
            BatchSize::SmallInput,
        )
    });

    c.bench_function("true simple 3", |b| {
        b.iter_batched(
            || Puzzle::from_sgf(include_str!("../src/test_sgfs/puzzles/true_simple3.sgf")),
            |mut puzzle| puzzle.solve(),
            BatchSize::SmallInput,
        )
    });

    let mut medium = c.benchmark_group("medium");
    medium.sample_size(10);

    medium.bench_function("true medium 1", |b| {
        b.iter_batched(
            || Puzzle::from_sgf(include_str!("../src/test_sgfs/puzzles/true_medium1.sgf")),
            |mut puzzle| puzzle.solve(),
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

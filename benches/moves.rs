use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tsumego_solver::go::{BoardPosition, GoGame};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("simple move", |b| {
        let game = GoGame::from_sgf(include_str!("../src/test_sgfs/ko_rule_simple.sgf"));

        b.iter(|| game.play_move(black_box(BoardPosition::new(4, 3))))
    });

    c.bench_function("complex capture", |b| {
        let game = GoGame::from_sgf(include_str!("../src/test_sgfs/complex_capture.sgf"));

        b.iter(|| game.play_move(black_box(BoardPosition::new(11, 6))))
    });

    c.bench_function("generating all moves", |b| {
        let game = GoGame::from_sgf(include_str!("../src/test_sgfs/puzzles/true_simple1.sgf"));

        b.iter(|| game.generate_moves())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

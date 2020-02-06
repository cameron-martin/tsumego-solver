use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tsumego_solver::go::GoGame;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("simple move", |b| {
        let game = GoGame::from_sgf(include_str!("../src/test_sgfs/ko_rule_simple.sgf"));

        b.iter(|| game.play_move(black_box((4, 3))))
    });

    c.bench_function("complex capture", |b| {
        let game = GoGame::from_sgf(include_str!("../src/test_sgfs/complex_capture.sgf"));

        b.iter(|| game.play_move(black_box((11, 6))))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

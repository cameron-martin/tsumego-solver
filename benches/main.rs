use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use tsumego_solver::go::{BoardPosition, GoGame, Move};
use tsumego_solver::puzzle::NoProfile;
use tsumego_solver::puzzle::Puzzle;

fn playing_moves(c: &mut Criterion) {
    let mut group = c.benchmark_group("playing moves");

    group.bench_function("simple move", |b| {
        let game = GoGame::from_sgf(include_str!("../src/test_sgfs/ko_rule_simple.sgf"));

        b.iter(|| game.play_move(black_box(Move::Place(BoardPosition::new(4, 3)))))
    });

    group.bench_function("complex capture", |b| {
        let game = GoGame::from_sgf(include_str!("../src/test_sgfs/complex_capture.sgf"));

        b.iter(|| game.play_move(black_box(Move::Place(BoardPosition::new(11, 6)))))
    });

    group.bench_function("generating all moves", |b| {
        let game = GoGame::from_sgf(include_str!("../src/test_sgfs/puzzles/true_simple1.sgf"));

        b.iter(|| game.generate_moves())
    });
}

fn unconditional_life(c: &mut Criterion) {
    let mut group = c.benchmark_group("unconditional life");

    group.bench_function("mixture", |b| {
        let game = GoGame::from_sgf(include_str!("../src/test_sgfs/life_and_death/mixture.sgf"));

        b.iter(|| game.get_board().unconditionally_alive_blocks())
    });

    group.bench_function("all alive 1", |b| {
        let game = GoGame::from_sgf(include_str!(
            "../src/test_sgfs/life_and_death/all_alive1.sgf"
        ));

        b.iter(|| game.get_board().unconditionally_alive_blocks())
    });

    group.bench_function("all dead 1", |b| {
        let game = GoGame::from_sgf(include_str!(
            "../src/test_sgfs/life_and_death/all_dead1.sgf"
        ));

        b.iter(|| game.get_board().unconditionally_alive_blocks())
    });
}

fn solving_puzzles(c: &mut Criterion) {
    let mut simple = c.benchmark_group("solving puzzles (simple)");

    simple.bench_function("1", |b| {
        b.iter_batched(
            || {
                Puzzle::<NoProfile>::from_sgf(include_str!(
                    "../src/test_sgfs/puzzles/true_simple1.sgf"
                ))
            },
            |mut puzzle| puzzle.solve(),
            BatchSize::SmallInput,
        )
    });

    simple.bench_function("2", |b| {
        b.iter_batched(
            || {
                Puzzle::<NoProfile>::from_sgf(include_str!(
                    "../src/test_sgfs/puzzles/true_simple2.sgf"
                ))
            },
            |mut puzzle| puzzle.solve(),
            BatchSize::SmallInput,
        )
    });

    simple.bench_function("3", |b| {
        b.iter_batched(
            || {
                Puzzle::<NoProfile>::from_sgf(include_str!(
                    "../src/test_sgfs/puzzles/true_simple3.sgf"
                ))
            },
            |mut puzzle| puzzle.solve(),
            BatchSize::SmallInput,
        )
    });

    simple.finish();

    let mut medium = c.benchmark_group("solving puzzles (medium)");
    medium.sample_size(10);

    medium.bench_function("1", |b| {
        b.iter_batched(
            || {
                Puzzle::<NoProfile>::from_sgf(include_str!(
                    "../src/test_sgfs/puzzles/true_medium1.sgf"
                ))
            },
            |mut puzzle| puzzle.solve(),
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, playing_moves, unconditional_life, solving_puzzles);
criterion_main!(benches);

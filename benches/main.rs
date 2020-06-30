use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use std::{path::Path, rc::Rc};
use tsumego_solver::go::{BoardPosition, GoGame, GoPlayer, Move};
use tsumego_solver::puzzle::NoProfile;
use tsumego_solver::puzzle::{CnnMoveRanker, MoveRanker, NullExampleCollector, Puzzle};

fn playing_moves(c: &mut Criterion) {
    let mut group = c.benchmark_group("playing moves");

    group.bench_function("simple move", |b| {
        b.iter_batched(
            || {
                GoGame::from_sgf(
                    include_str!("../src/test_sgfs/ko_rule_simple.sgf"),
                    GoPlayer::Black,
                )
            },
            |game| game.play_move(black_box(Move::Place(BoardPosition::new(4, 3)))),
            BatchSize::SmallInput,
        )
    });

    group.bench_function("complex capture", |b| {
        b.iter_batched(
            || {
                GoGame::from_sgf(
                    include_str!("../src/test_sgfs/complex_capture.sgf"),
                    GoPlayer::Black,
                )
            },
            |game| game.play_move(black_box(Move::Place(BoardPosition::new(11, 6)))),
            BatchSize::SmallInput,
        )
    });

    group.bench_function("generating all moves", |b| {
        b.iter_batched(
            || {
                GoGame::from_sgf(
                    include_str!("../src/test_sgfs/puzzles/true_simple1.sgf"),
                    GoPlayer::Black,
                )
            },
            |game| game.generate_moves().collect::<Vec<_>>(),
            BatchSize::SmallInput,
        )
    });
}

fn unconditional_life(c: &mut Criterion) {
    let mut group = c.benchmark_group("unconditional life");

    group.bench_function("mixture", |b| {
        b.iter_batched(
            || {
                GoGame::from_sgf(
                    include_str!("../src/test_sgfs/life_and_death/mixture.sgf"),
                    GoPlayer::Black,
                )
            },
            |game| game.board.unconditionally_alive_blocks(),
            BatchSize::SmallInput,
        )
    });

    group.bench_function("all alive 1", |b| {
        b.iter_batched(
            || {
                GoGame::from_sgf(
                    include_str!("../src/test_sgfs/life_and_death/all_alive1.sgf"),
                    GoPlayer::Black,
                )
            },
            |game| game.board.unconditionally_alive_blocks(),
            BatchSize::SmallInput,
        )
    });

    group.bench_function("all dead 1", |b| {
        b.iter_batched(
            || {
                GoGame::from_sgf(
                    include_str!("../src/test_sgfs/life_and_death/all_dead1.sgf"),
                    GoPlayer::Black,
                )
            },
            |game| game.board.unconditionally_alive_blocks(),
            BatchSize::SmallInput,
        )
    });
}

fn ordering_moves(c: &mut Criterion) {
    let mut group = c.benchmark_group("ordering moves");

    let move_ranker = Rc::new(CnnMoveRanker::new(
        &Path::new(file!())
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("network")
            .join("model"),
    ));

    group.bench_function("doing the ordering", |b| {
        b.iter_batched(
            || {
                GoGame::from_sgf(
                    include_str!("../src/test_sgfs/puzzles/true_ultrasimple1.sgf"),
                    GoPlayer::Black,
                )
            },
            |game| move_ranker.order_moves(game),
            BatchSize::SmallInput,
        )
    });

    // group.bench_function("generating ordered moves", |b| {
    //     b.iter_batched(
    //         || {
    //             GoGame::from_sgf(include_str!(
    //                 "../src/test_sgfs/puzzles/true_ultrasimple1.sgf"
    //             ))
    //         },
    //         |game| {
    //             game.generate_ordered_moves(move_ranker.order_moves(game))
    //                 .collect::<Vec<_>>()
    //         },
    //         BatchSize::SmallInput,
    //     )
    // });
}

fn solving_puzzles(c: &mut Criterion) {
    let mut ultra_simple = c.benchmark_group("solving puzzles (ultrasimple)");

    let move_ranker = Rc::new(CnnMoveRanker::new(
        &Path::new(file!())
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("network")
            .join("model"),
    ));

    ultra_simple.bench_function("1", |b| {
        b.iter_batched(
            || {
                Puzzle::from_sgf(
                    include_str!("../src/test_sgfs/puzzles/true_ultrasimple1.sgf"),
                    GoPlayer::Black,
                )
            },
            |puzzle| {
                puzzle.solve::<NoProfile, _, _>(&mut NullExampleCollector, move_ranker.clone())
            },
            BatchSize::SmallInput,
        )
    });

    ultra_simple.bench_function("2", |b| {
        b.iter_batched(
            || {
                Puzzle::from_sgf(
                    include_str!("../src/test_sgfs/puzzles/true_ultrasimple2.sgf"),
                    GoPlayer::Black,
                )
            },
            |puzzle| {
                puzzle.solve::<NoProfile, _, _>(&mut NullExampleCollector, move_ranker.clone())
            },
            BatchSize::SmallInput,
        )
    });

    ultra_simple.finish();

    let mut simple = c.benchmark_group("solving puzzles (simple)");

    simple.bench_function("1", |b| {
        b.iter_batched(
            || {
                Puzzle::from_sgf(
                    include_str!("../src/test_sgfs/puzzles/true_simple1.sgf"),
                    GoPlayer::Black,
                )
            },
            |puzzle| {
                puzzle.solve::<NoProfile, _, _>(&mut NullExampleCollector, move_ranker.clone())
            },
            BatchSize::SmallInput,
        )
    });

    simple.bench_function("2", |b| {
        b.iter_batched(
            || {
                Puzzle::from_sgf(
                    include_str!("../src/test_sgfs/puzzles/true_simple2.sgf"),
                    GoPlayer::Black,
                )
            },
            |puzzle| {
                puzzle.solve::<NoProfile, _, _>(&mut NullExampleCollector, move_ranker.clone())
            },
            BatchSize::SmallInput,
        )
    });

    simple.bench_function("3", |b| {
        b.iter_batched(
            || {
                Puzzle::from_sgf(
                    include_str!("../src/test_sgfs/puzzles/true_simple3.sgf"),
                    GoPlayer::Black,
                )
            },
            |puzzle| {
                puzzle.solve::<NoProfile, _, _>(&mut NullExampleCollector, move_ranker.clone())
            },
            BatchSize::SmallInput,
        )
    });

    simple.finish();

    let mut medium = c.benchmark_group("solving puzzles (medium)");
    medium.sample_size(10);

    medium.bench_function("1", |b| {
        b.iter_batched(
            || {
                Puzzle::from_sgf(
                    include_str!("../src/test_sgfs/puzzles/true_medium1.sgf"),
                    GoPlayer::Black,
                )
            },
            |puzzle| {
                puzzle.solve::<NoProfile, _, _>(&mut NullExampleCollector, move_ranker.clone())
            },
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(
    benches,
    playing_moves,
    unconditional_life,
    ordering_moves,
    solving_puzzles
);
criterion_main!(benches);

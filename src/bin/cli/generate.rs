use std::{
    fs::{self, OpenOptions},
    io,
    path::Path,
    rc::Rc,
    sync::mpsc::channel,
    thread,
    time::Duration,
};
use tsumego_solver::puzzle::ExampleCollector;
use tsumego_solver::{
    generation::generate_puzzle,
    go::{GoGame, GoPlayer},
    puzzle::{
        ChannelExampleCollector, CnnMoveRanker, FileExampleCollector, MoveRanker, NoProfile,
        NullExampleCollector, Profile, Puzzle, RandomMoveRanker,
    },
};

pub fn run(output_directory: &Path, thread_count: u8, model_dir: &str) -> io::Result<()> {
    fs::create_dir_all(output_directory)?;

    let (puzzle_tx, puzzle_rx) = channel();
    let (examples_tx, examples_rx) = channel();

    let example_collector = ChannelExampleCollector::new(examples_tx);

    for _ in 0..thread_count {
        let puzzle_tx = puzzle_tx.clone();
        let mut example_collector = example_collector.clone();
        let model_dir = String::from(model_dir);

        thread::spawn(move || {
            // let move_ranker = Rc::new(CnnMoveRanker::new(Path::new(&model_dir)));
            let move_ranker = Rc::new(RandomMoveRanker);

            loop {
                let generated_puzzle = generate_puzzle::<Profile, _, _>(
                    Duration::from_secs(1),
                    &mut example_collector,
                    move_ranker.clone(),
                );

                // Re-solve, collecting examples. This ensures that only examples are collected from sensible puzzles
                // for &player in GoPlayer::both() {
                //     let puzzle = Puzzle::new(GoGame::from_board(generated_puzzle.board, player));

                //     puzzle.solve::<NoProfile, _, _>(&mut example_collector, move_ranker.clone());
                // }

                puzzle_tx.send(generated_puzzle).unwrap();
            }
        });
    }

    {
        let output_directory = output_directory.to_owned();

        thread::spawn(move || {
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(output_directory.join("_examples.bin"))
                .unwrap();

            let mut example_collector = FileExampleCollector::new(file, 128);

            loop {
                let (game, go_move) = examples_rx.recv().unwrap();

                example_collector.collect_example(game, go_move);
            }
        });
    }

    loop {
        let puzzle = puzzle_rx.recv().unwrap();

        let file = output_directory.join(format!("{:016x}.sgf", puzzle.board.stable_hash()));
        if file.exists() {
            println!("Duplicate {}", file.display());
        } else {
            fs::write(file.as_path(), puzzle.board.to_sgf())?;

            println!(
                "Generated {}:\n  White profile:\n{}\n  Black profile:\n{}\n",
                file.display(),
                textwrap::indent(&format!("{}", puzzle.white_solution.profiler), "    "),
                textwrap::indent(&format!("{}", puzzle.black_solution.profiler), "    ")
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;
    use tsumego_solver::{
        go::{BoardPosition, GoBoard, Move},
        puzzle::{NoProfile, Solution},
    };

    // #[test]
    // fn extract_examples_test() {
    //     let puzzle = GoBoard::empty();

    //     let solution = Solution {
    //         won: true,
    //         principle_variation: vec![
    //             Move::Place(BoardPosition::new(0, 0)),
    //             Move::Place(BoardPosition::new(1, 0)),
    //             Move::Place(BoardPosition::new(0, 1)),
    //             Move::Place(BoardPosition::new(1, 1)),
    //         ],
    //         profiler: NoProfile,
    //     };

    //     let examples = extract_examples(puzzle, &solution, GoPlayer::Black);

    //     let mut snapshot = String::new();

    //     for (board, go_move) in examples {
    //         snapshot.push_str(&format!("{}\n{}\n\n", go_move, board));
    //     }

    //     assert_snapshot!(snapshot);
    // }
}

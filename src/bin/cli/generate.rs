use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io,
    io::Write,
    path::Path,
    sync::mpsc::channel,
    thread,
    time::Duration,
};
use tsumego_solver::{
    generation::generate_puzzle,
    go::{GoBoard, GoGame, GoPlayer, Move},
    puzzle::{NoProfile, Solution},
};

fn extract_examples(
    puzzle: GoBoard,
    solution: &Solution<NoProfile>,
    first_player: GoPlayer,
) -> Vec<(GoBoard, Move)> {
    let game = GoGame::from_board(puzzle, first_player);

    solution
        .principle_variation
        .iter()
        .scan(game, |game, &go_move| {
            let board = if game.current_player == GoPlayer::White {
                game.board.invert_colours()
            } else {
                game.board
            };
            let example = (board, go_move);

            *game = game.play_move(go_move).unwrap();

            Some(example)
        })
        .collect()
}

pub fn run(output_directory: &Path, thread_count: u8) -> io::Result<()> {
    fs::create_dir_all(output_directory)?;

    let (tx, rx) = channel();

    for _ in 0..thread_count {
        let tx = tx.clone();
        thread::spawn(move || loop {
            let puzzle = generate_puzzle::<NoProfile>(Duration::from_secs(1));
            tx.send(puzzle).unwrap();
        });
    }

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(output_directory.join("_examples.bin"))?;

    loop {
        let puzzle = rx.recv().unwrap();

        for &player in GoPlayer::both() {
            let examples =
                extract_examples(puzzle.board, puzzle.solution_for_player(player), player);
            for (board, go_move) in examples {
                let bytes = [
                    board.get_bitboard_for_player(GoPlayer::Black).serialise(),
                    board.get_bitboard_for_player(GoPlayer::White).serialise(),
                    go_move.serialise()
                ].concat();
                file.write_all(bytes);
            }
        }

        // puzzle.black_solution.principle_variation
        // let example = HashMap::new();
        // example.insert("board");
        // example.insert("move", );

        // writer.send(example);

        let file = output_directory.join(format!("{:016x}.sgf", puzzle.board.stable_hash()));
        if file.exists() {
            println!("Duplicate {}", file.display());
        } else {
            fs::write(file.as_path(), puzzle.board.to_sgf())?;

            println!("Generated {}", file.display());
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

    #[test]
    fn extract_examples_test() {
        let puzzle = GoBoard::empty();

        let solution = Solution {
            won: true,
            principle_variation: vec![
                Move::Place(BoardPosition::new(0, 0)),
                Move::Place(BoardPosition::new(1, 0)),
                Move::Place(BoardPosition::new(0, 1)),
                Move::Place(BoardPosition::new(1, 1)),
            ],
            profiler: NoProfile,
        };

        let examples = extract_examples(puzzle, &solution, GoPlayer::Black);

        let mut snapshot = String::new();

        for (board, go_move) in examples {
            snapshot.push_str(&format!("{}\n{}\n\n", go_move, board));
        }

        assert_snapshot!(snapshot);
    }
}

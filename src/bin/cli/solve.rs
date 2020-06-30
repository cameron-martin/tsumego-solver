use std::{
    fs::{self, OpenOptions},
    io,
    path::Path,
    rc::Rc,
};

use tsumego_solver::{
    go::{GoGame, GoPlayer},
    puzzle::{FileExampleCollector, LinearMoveRanker, NoProfile, Puzzle},
};

pub fn run(dir: &Path, model_dir: &str) -> io::Result<()> {
    let examples_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("_examples.bin"))
        .unwrap();

    let mut example_collector = FileExampleCollector::new(examples_file, 128);

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext != "sgf" {
                continue;
            }
        }

        let sgf_file = fs::read_to_string(&path)?;

        for &player in GoPlayer::both() {
            let game = GoGame::from_sgf(&sgf_file, player);
            let puzzle = Puzzle::new(game);

            puzzle.solve::<NoProfile, _, _>(&mut example_collector, Rc::new(LinearMoveRanker));
        }

        println!("Solved {}", path.display());
    }

    Ok(())
}

use std::fs;
use std::io;
use std::path::Path;
use std::time::Duration;
use tsumego_solver::generation::generate_puzzle;
use uuid::Uuid;

pub fn run(output_directory: &Path) -> io::Result<()> {
    fs::create_dir_all(output_directory)?;

    loop {
        let puzzle = generate_puzzle(Duration::from_secs(1));
        let file = output_directory.join(format!("{}.sgf", Uuid::new_v4()));
        fs::write(file.as_path(), puzzle.to_sgf())?;

        println!("Generated {}", file.display());
    }
}

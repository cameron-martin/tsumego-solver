extern crate pest;

use std::fs;
use std::path::Path;
use tsumego_solver::{
    gotools_parser,
    puzzle::{LinearMoveRanker, NoProfile, NullExampleCollector},
};

use gotools_parser::PuzzleCollection;
use std::borrow::Borrow;
use std::{error::Error, rc::Rc, time::Duration};

fn read_puzzles() -> Result<PuzzleCollection, Box<dyn Error>> {
    let mut puzzles = PuzzleCollection::new();
    let dir = Path::new(file!()).parent().unwrap().join("puzzles");
    for file in fs::read_dir(dir)? {
        let path = file?.path();
        let contents = fs::read_to_string(&path)?;
        puzzles.append(gotools_parser::parse(contents.borrow())?);
    }
    Ok(puzzles)
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut puzzles = read_puzzles()?;

    let mut solved_count = 0;

    for puzzle in puzzles.valid_puzzles.iter_mut() {
        if let Some(_solution) = puzzle.solve_with_timeout::<NoProfile, _, _>(
            Duration::from_millis(10),
            &mut NullExampleCollector,
            Rc::new(LinearMoveRanker),
        ) {
            solved_count += 1;
        }
    }

    println!(
        "Total Count: {}\nValid Count: {}\nSolved in 10ms: {}",
        puzzles.total_puzzles,
        puzzles.valid_puzzles.len(),
        solved_count
    );

    Ok(())
}

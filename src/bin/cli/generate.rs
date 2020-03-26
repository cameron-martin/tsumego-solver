use std::time::Duration;
use tsumego_solver::generation::generate_puzzle;

pub fn run() {
    let puzzle = generate_puzzle(Duration::from_secs(1));

    println!("{}", puzzle);
}

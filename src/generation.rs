mod candidate;
mod validation;

use crate::go::{GoBoard, GoPlayer};
use crate::puzzle::{Profiler, Solution};
pub use candidate::generate_candidate;
use std::time::Duration;
pub use validation::validate_candidate;

pub struct GeneratedPuzzle<P: Profiler> {
    pub board: GoBoard,
    pub white_solution: Solution<P>,
    pub black_solution: Solution<P>,
}

impl<P: Profiler> GeneratedPuzzle<P> {
    pub fn solution_for_player(&self, player: GoPlayer) -> &Solution<P> {
        match player {
            GoPlayer::White => &self.white_solution,
            GoPlayer::Black => &self.black_solution,
        }
    }
}

pub fn generate_puzzle<P: Profiler>(timeout: Duration) -> GeneratedPuzzle<P> {
    let mut rng = rand::thread_rng();

    loop {
        let candidate = generate_candidate(&mut rng);

        if let Some((white_solution, black_solution)) = validate_candidate::<P>(candidate, timeout)
        {
            return GeneratedPuzzle {
                board: candidate,
                white_solution,
                black_solution,
            };
        }
    }
}

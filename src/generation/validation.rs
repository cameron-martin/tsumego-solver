use crate::go::{GoBoard, GoGame, GoPlayer};
use crate::puzzle::{Profiler, Puzzle, Solution};
use std::time::Duration;

pub fn validate_candidate<P: Profiler>(
    candidate: GoBoard,
    timeout: Duration,
) -> Option<(Solution<P>, Solution<P>)> {
    if candidate.has_captured_groups() {
        return None;
    }

    let solve_puzzle = |player: GoPlayer| {
        Puzzle::new(GoGame::from_board(candidate, player)).solve_with_timeout::<P>(timeout)
    };

    if let Some(white_solution) = solve_puzzle(GoPlayer::White) {
        if let Some(black_solution) = solve_puzzle(GoPlayer::Black) {
            if white_solution.won && black_solution.won {
                return Some((white_solution, black_solution));
            }
        }
    }

    None
}

use crate::go::{GoBoard, GoGame, GoPlayer};
use crate::puzzle::{ExampleCollector, MoveRanker, Profiler, Puzzle, Solution};
use std::{rc::Rc, time::Duration};

pub fn validate_candidate<P: Profiler, E: ExampleCollector, R: MoveRanker>(
    candidate: GoBoard,
    timeout: Duration,
    example_collector: &mut E,
    move_ranker: Rc<R>,
) -> Option<(Solution<P>, Solution<P>)> {
    if candidate.has_captured_groups() {
        return None;
    }

    let mut solve_puzzle = |player: GoPlayer| {
        Puzzle::new(GoGame::from_board(candidate, player)).solve_with_timeout::<P, _, _>(
            timeout,
            example_collector,
            move_ranker.clone(),
        )
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

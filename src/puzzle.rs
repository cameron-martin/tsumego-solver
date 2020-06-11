mod abort_controller;
mod profiler;
mod solution;
mod solving_iteration;
mod solving_session;
mod terminal_detection;

use crate::go::{GoGame, GoPlayer};
use abort_controller::{AbortController, NoAbortController, TimeoutAbortController};
pub use profiler::{NoProfile, Profile, Profiler};
pub use solution::Solution;
use solving_session::SolvingSession;
use std::time::Duration;

#[derive(Copy, Clone)]
pub struct Puzzle {
    pub game: GoGame,
    pub player: GoPlayer,
    pub attacker: GoPlayer,
}

impl Puzzle {
    pub fn new(game: GoGame) -> Puzzle {
        let attacker = if !(game.board.out_of_bounds().expand_one()
            & game.board.get_bitboard_for_player(GoPlayer::White))
        .is_empty()
        {
            GoPlayer::White
        } else {
            GoPlayer::Black
        };

        let player = game.current_player;

        Puzzle {
            game,
            player,
            attacker,
        }
    }

    pub fn from_sgf(sgf_string: &str) -> Puzzle {
        Self::new(GoGame::from_sgf(sgf_string))
    }

    pub fn solve<P: Profiler>(&self) -> Solution<P> {
        self.solve_with_controller::<_, P>(NoAbortController)
            .unwrap()
    }

    pub fn solve_with_timeout<P: Profiler>(&self, timeout: Duration) -> Option<Solution<P>> {
        self.solve_with_controller::<_, P>(TimeoutAbortController::duration(timeout))
    }

    fn solve_with_controller<C: AbortController, P: Profiler>(
        &self,
        abort_controller: C,
    ) -> Option<Solution<P>> {
        let mut max_depth: u8 = 1;

        let mut session = SolvingSession::new(*self, abort_controller);

        loop {
            let mut iteration = session.create_iteration(max_depth);
            let result = iteration.solve()?;

            if result != 0 {
                return Some(Solution {
                    won: result > 0,
                    principle_variation: iteration.principle_variation(),
                    profiler: session.profiler,
                });
            }

            max_depth += 1;
            session.profiler.move_down();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::go::GoGame;
    use insta::{assert_display_snapshot, assert_snapshot};
    use profiler::Profile;
    use std::borrow::Borrow;

    fn show_principle_variation<P: Profiler>(puzzle: &Puzzle, solution: &Solution<P>) -> String {
        let games = solution
            .principle_variation
            .iter()
            .scan(puzzle.game, |state, &go_move| {
                *state = state.play_move(go_move).unwrap();

                Some((*state, go_move))
            });

        let mut output = String::new();
        output.push_str(&format!("{}\n\n", puzzle.game.board));
        for (game, go_move) in games {
            output.push_str(
                format!(
                    "{}: {}\n{}\n\n",
                    game.current_player.flip(),
                    go_move,
                    game.board
                )
                .borrow(),
            );
        }

        output
    }

    #[test]
    fn true_simple1() {
        let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_simple1.sgf"));

        let puzzle = Puzzle::new(tsumego);

        let solution = puzzle.solve::<Profile>();

        assert!(solution.won);
        assert_display_snapshot!(solution.profiler.visited_nodes, @"644");
        assert_display_snapshot!(solution.profiler.max_depth, @"5");
        assert_snapshot!(show_principle_variation(&puzzle, &solution));
    }

    #[test]
    fn true_simple2() {
        let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_simple2.sgf"));

        let puzzle = Puzzle::new(tsumego);

        let solution = puzzle.solve::<Profile>();

        assert!(solution.won);
        assert_display_snapshot!(solution.profiler.visited_nodes, @"1657");
        assert_display_snapshot!(solution.profiler.max_depth, @"7");
        assert_snapshot!(show_principle_variation(&puzzle, &solution));
    }

    #[test]
    fn true_simple3() {
        let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_simple3.sgf"));

        let puzzle = Puzzle::new(tsumego);

        let solution = puzzle.solve::<Profile>();

        assert!(solution.won);
        assert_display_snapshot!(solution.profiler.visited_nodes, @"1490");
        assert_display_snapshot!(solution.profiler.max_depth, @"8");
        assert_snapshot!(show_principle_variation(&puzzle, &solution));
    }

    #[test]
    fn true_simple4() {
        let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_simple4.sgf"));

        let puzzle = Puzzle::new(tsumego);

        let solution = puzzle.solve::<Profile>();

        assert!(solution.won);
        assert_display_snapshot!(solution.profiler.visited_nodes, @"31143");
        assert_display_snapshot!(solution.profiler.max_depth, @"7");
        assert_snapshot!(show_principle_variation(&puzzle, &solution));
    }

    // #[test]
    // fn true_medium1() {
    //     let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_medium1.sgf"));

    //     let mut puzzle = Puzzle::<Profile>::new(tsumego);

    //     let solution = puzzle.solve();

    //     assert!(solution.won);
    //     // assert_eq!(puzzle.first_move(), Move::Place(BoardPosition::new(14, 2)));
    //     assert_display_snapshot!(puzzle.profiler.visited_nodes, @"345725");
    //     assert_display_snapshot!(puzzle.profiler.max_depth, @"29");
    // }

    #[test]
    fn true_ultrasimple1() {
        let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_ultrasimple1.sgf"));

        let puzzle = Puzzle::new(tsumego);

        let solution = puzzle.solve::<Profile>();

        assert!(solution.won);
        assert_display_snapshot!(solution.profiler.visited_nodes, @"5");
        assert_display_snapshot!(solution.profiler.max_depth, @"1");
        assert_snapshot!(show_principle_variation(&puzzle, &solution));
    }

    #[test]
    fn true_ultrasimple2() {
        let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_ultrasimple2.sgf"));

        let puzzle = Puzzle::new(tsumego);

        let solution = puzzle.solve::<Profile>();

        assert!(solution.won);
        assert_display_snapshot!(solution.profiler.visited_nodes, @"1939");
        assert_display_snapshot!(solution.profiler.max_depth, @"8");
        assert_snapshot!(show_principle_variation(&puzzle, &solution));
    }
}

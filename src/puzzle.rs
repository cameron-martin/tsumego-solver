mod abort_controller;
mod example_collector;
mod move_ranker;
mod profiler;
mod solution;
mod solving_iteration;
mod solving_session;
mod terminal_detection;

use crate::go::{GoGame, GoPlayer};
use abort_controller::{AbortController, NoAbortController, TimeoutAbortController};
pub use example_collector::{
    ChannelExampleCollector, ExampleCollector, FileExampleCollector, NullExampleCollector,
};
pub use move_ranker::{CnnMoveRanker, LinearMoveRanker, MoveRanker, RandomMoveRanker};
pub use profiler::{NoProfile, Profile, Profiler};
pub use solution::Solution;
use solving_session::SolvingSession;
use std::{rc::Rc, time::Duration};

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

    pub fn from_sgf(sgf_string: &str, first_player: GoPlayer) -> Puzzle {
        Self::new(GoGame::from_sgf(sgf_string, first_player))
    }

    pub fn solve<P: Profiler, E: ExampleCollector, R: MoveRanker>(
        &self,
        example_collector: &mut E,
        move_ranker: Rc<R>,
    ) -> Solution<P> {
        self.solve_with_controller::<_, P, _, _>(NoAbortController, example_collector, move_ranker)
            .unwrap()
    }

    pub fn solve_with_timeout<P: Profiler, E: ExampleCollector, R: MoveRanker>(
        &self,
        timeout: Duration,
        example_collector: &mut E,
        move_ranker: Rc<R>,
    ) -> Option<Solution<P>> {
        self.solve_with_controller::<_, P, _, _>(
            TimeoutAbortController::duration(timeout),
            example_collector,
            move_ranker,
        )
    }

    fn solve_with_controller<
        C: AbortController,
        P: Profiler,
        E: ExampleCollector,
        R: MoveRanker,
    >(
        &self,
        abort_controller: C,
        example_collector: &mut E,
        move_ranker: Rc<R>,
    ) -> Option<Solution<P>> {
        let mut max_depth: u8 = 1;

        let mut session =
            SolvingSession::new(*self, abort_controller, example_collector, move_ranker);

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
    use move_ranker::LinearMoveRanker;
    use profiler::Profile;
    use std::{borrow::Borrow, path::Path, rc::Rc};

    fn create_principal_move_ranker() -> impl MoveRanker {
        CnnMoveRanker::new(Path::new("network/model"))
        // LinearMoveRanker
        // RandomMoveRanker
    }

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
    fn generated_0a0c446484626b51() {
        let puzzle = Puzzle::from_sgf(
            include_str!("test_sgfs/puzzles/generated_0a0c446484626b51.sgf"),
            GoPlayer::Black,
        );

        let solution = puzzle.solve::<Profile, _, _>(
            &mut NullExampleCollector,
            Rc::new(create_principal_move_ranker()),
        );

        assert!(solution.won);
        assert_display_snapshot!(solution.profiler.ordering_accuracy(), @"0");
        assert_display_snapshot!(solution.profiler.visited_nodes, @"380");
        assert_display_snapshot!(solution.profiler.max_depth, @"5");
        assert_snapshot!(show_principle_variation(&puzzle, &solution));
    }

    #[test]
    fn generated_00a1c46fb85ba4a7() {
        let puzzle = Puzzle::from_sgf(
            include_str!("test_sgfs/puzzles/generated_00a1c46fb85ba4a7.sgf"),
            GoPlayer::Black,
        );

        let solution = puzzle.solve::<Profile, _, _>(
            &mut NullExampleCollector,
            Rc::new(create_principal_move_ranker()),
        );

        assert!(solution.won);
        assert_display_snapshot!(solution.profiler.ordering_accuracy(), @"0");
        assert_display_snapshot!(solution.profiler.visited_nodes, @"1391");
        assert_display_snapshot!(solution.profiler.max_depth, @"5");
        assert_snapshot!(show_principle_variation(&puzzle, &solution));
    }

    #[test]
    fn generated_0a2cae535ebf01be() {
        let puzzle = Puzzle::from_sgf(
            include_str!("test_sgfs/puzzles/generated_0a2cae535ebf01be.sgf"),
            GoPlayer::Black,
        );

        let solution = puzzle.solve::<Profile, _, _>(
            &mut NullExampleCollector,
            Rc::new(create_principal_move_ranker()),
        );

        assert!(solution.won);
        assert_display_snapshot!(solution.profiler.ordering_accuracy(), @"0");
        assert_display_snapshot!(solution.profiler.visited_nodes, @"140057");
        assert_display_snapshot!(solution.profiler.max_depth, @"9");
        assert_snapshot!(show_principle_variation(&puzzle, &solution));
    }

    #[test]
    fn true_simple1() {
        let puzzle = Puzzle::from_sgf(
            include_str!("test_sgfs/puzzles/true_simple1.sgf"),
            GoPlayer::Black,
        );

        let solution = puzzle.solve::<Profile, _, _>(
            &mut NullExampleCollector,
            Rc::new(create_principal_move_ranker()),
        );

        assert!(solution.won);
        assert_display_snapshot!(solution.profiler.ordering_accuracy(), @"0");
        assert_display_snapshot!(solution.profiler.visited_nodes, @"505");
        assert_display_snapshot!(solution.profiler.max_depth, @"5");
        assert_snapshot!(show_principle_variation(&puzzle, &solution));
    }

    #[test]
    fn true_simple2() {
        let puzzle = Puzzle::from_sgf(
            include_str!("test_sgfs/puzzles/true_simple2.sgf"),
            GoPlayer::Black,
        );

        let solution = puzzle.solve::<Profile, _, _>(
            &mut NullExampleCollector,
            Rc::new(create_principal_move_ranker()),
        );

        assert!(solution.won);
        assert_display_snapshot!(solution.profiler.ordering_accuracy(), @"0");
        assert_display_snapshot!(solution.profiler.visited_nodes, @"1558");
        assert_display_snapshot!(solution.profiler.max_depth, @"7");
        assert_snapshot!(show_principle_variation(&puzzle, &solution));
    }

    #[test]
    fn true_simple3() {
        let puzzle = Puzzle::from_sgf(
            include_str!("test_sgfs/puzzles/true_simple3.sgf"),
            GoPlayer::Black,
        );

        let solution = puzzle.solve::<Profile, _, _>(
            &mut NullExampleCollector,
            Rc::new(create_principal_move_ranker()),
        );

        assert!(solution.won);
        assert_display_snapshot!(solution.profiler.ordering_accuracy(), @"0");
        assert_display_snapshot!(solution.profiler.visited_nodes, @"691");
        assert_display_snapshot!(solution.profiler.max_depth, @"8");
        assert_snapshot!(show_principle_variation(&puzzle, &solution));
    }

    #[test]
    fn true_simple4() {
        let puzzle = Puzzle::from_sgf(
            include_str!("test_sgfs/puzzles/true_simple4.sgf"),
            GoPlayer::Black,
        );

        let solution = puzzle.solve::<Profile, _, _>(
            &mut NullExampleCollector,
            Rc::new(create_principal_move_ranker()),
        );

        assert!(solution.won);
        assert_display_snapshot!(solution.profiler.ordering_accuracy(), @"0");
        assert_display_snapshot!(solution.profiler.visited_nodes, @"24209");
        assert_display_snapshot!(solution.profiler.max_depth, @"7");
        assert_snapshot!(show_principle_variation(&puzzle, &solution));
    }

    // #[test]
    // fn true_medium1() {
    //     let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_medium1.sgf"));

    //     let mut puzzle = Puzzle::new(tsumego);

    //     let solution =
    //         puzzle.solve::<Profile>(Rc::new(MoveRanker::new(Path::new("network/model"))));

    //     assert!(solution.won);
    //     assert_display_snapshot!(solution.profiler.visited_nodes, @"345725");
    //     assert_display_snapshot!(solution.profiler.max_depth, @"29");
    //     assert_snapshot!(show_principle_variation(&puzzle, &solution));
    // }

    #[test]
    fn true_ultrasimple1() {
        let puzzle = Puzzle::from_sgf(
            include_str!("test_sgfs/puzzles/true_ultrasimple1.sgf"),
            GoPlayer::Black,
        );

        let solution = puzzle.solve::<Profile, _, _>(
            &mut NullExampleCollector,
            Rc::new(create_principal_move_ranker()),
        );

        assert!(solution.won);
        assert_display_snapshot!(solution.profiler.ordering_accuracy(), @"0");
        assert_display_snapshot!(solution.profiler.visited_nodes, @"3");
        assert_display_snapshot!(solution.profiler.max_depth, @"1");
        assert_snapshot!(show_principle_variation(&puzzle, &solution));
    }

    #[test]
    fn true_ultrasimple2() {
        let puzzle = Puzzle::from_sgf(
            include_str!("test_sgfs/puzzles/true_ultrasimple2.sgf"),
            GoPlayer::Black,
        );

        let solution = puzzle.solve::<Profile, _, _>(
            &mut NullExampleCollector,
            Rc::new(create_principal_move_ranker()),
        );

        assert!(solution.won);
        assert_display_snapshot!(solution.profiler.ordering_accuracy(), @"0");
        assert_display_snapshot!(solution.profiler.visited_nodes, @"946");
        assert_display_snapshot!(solution.profiler.max_depth, @"8");
        assert_snapshot!(show_principle_variation(&puzzle, &solution));
    }
}

mod profiler;
mod terminal_detection;

use crate::go::{GoGame, GoPlayer};
pub use profiler::{NoProfile, Profile, Profiler};
use std::{
    cmp,
    collections::HashSet,
    time::{Duration, Instant},
};

trait AbortController {
    fn should_abort(&self) -> bool;
}

struct NoAbortController;

impl AbortController for NoAbortController {
    fn should_abort(&self) -> bool {
        false
    }
}

struct TimeoutAbortController {
    timeout_at: Instant,
}

impl AbortController for TimeoutAbortController {
    fn should_abort(&self) -> bool {
        Instant::now() >= self.timeout_at
    }
}

impl TimeoutAbortController {
    fn duration(duration: Duration) -> Self {
        TimeoutAbortController {
            timeout_at: Instant::now() + duration,
        }
    }
}

pub struct Puzzle<P: Profiler> {
    game: GoGame,
    player: GoPlayer,
    attacker: GoPlayer,
    pub profiler: P,
}

impl<P: Profiler> Puzzle<P> {
    pub fn new(game: GoGame) -> Puzzle<P> {
        let attacker = if !(game.get_board().out_of_bounds().expand_one()
            & game.get_board().get_bitboard_for_player(GoPlayer::White))
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
            profiler: P::new(),
        }
    }

    pub fn from_sgf(sgf_string: &str) -> Puzzle<P> {
        Self::new(GoGame::from_sgf(sgf_string))
    }

    pub fn solve(&self) -> bool {
        self.solve_with_controller(&NoAbortController).unwrap()
    }

    pub fn solve_with_timeout(&self, timeout: Duration) -> Option<bool> {
        self.solve_with_controller(&TimeoutAbortController::duration(timeout))
    }

    fn solve_with_controller<C: AbortController>(&self, controller: &C) -> Option<bool> {
        let mut parents = HashSet::new();
        parents.insert(self.game);

        let mut depth = 1;

        loop {
            let result = self.negamax(
                self.game,
                -std::i8::MAX,
                std::i8::MAX,
                depth,
                1,
                controller,
                &mut parents,
            )?;

            if result != 0 {
                return Some(result > 0);
            }

            depth += 1;
        }
    }

    fn negamax<C: AbortController>(
        &self,
        node: GoGame,
        a: i8,
        b: i8,
        depth: u8,
        is_maximising_player: i8,
        controller: &C,
        parents: &mut HashSet<GoGame>,
    ) -> Option<i8> {
        if controller.should_abort() {
            return None;
        }

        if depth == 0 {
            return Some(0);
        }

        if let Some(value) = terminal_detection::is_terminal(node, self.player, self.attacker) {
            return Some(is_maximising_player * if value { 1 } else { -1 });
        }

        let mut value = -std::i8::MAX;
        for (child, _move) in node.generate_moves_including_pass() {
            if parents.contains(&child) {
                continue;
            }
            parents.insert(child);
            value = cmp::max(
                value,
                -self.negamax(
                    child,
                    -b,
                    -a,
                    depth - 1,
                    -is_maximising_player,
                    controller,
                    parents,
                )?,
            );
            parents.remove(&child);
            let a = cmp::max(a, value);
            if a >= b {
                break;
            }
        }
        return Some(value);
    }

    // fn root_node(&self) -> AndOrNode {
    //     self.tree[self.root_id]
    // }

    // pub fn first_move(&self) -> Move {
    //     *self
    //         .tree
    //         .edges(self.root_id)
    //         .find(|edge| self.tree[edge.target()].is_proved())
    //         .unwrap()
    //         .weight()
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::go::{BoardPosition, GoGame};
    use insta::{assert_display_snapshot, assert_snapshot};
    use profiler::Profile;
    use std::borrow::Borrow;

    #[test]
    fn true_simple1() {
        let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_simple1.sgf"));

        let mut puzzle = Puzzle::<Profile>::new(tsumego);

        let won = puzzle.solve();

        assert!(won);
        // assert_eq!(puzzle.first_move(), Move::Place(BoardPosition::new(4, 0)));
        assert_display_snapshot!(puzzle.profiler.node_count, @"542");
        assert_display_snapshot!(puzzle.profiler.max_depth, @"7");
    }

    #[test]
    fn true_simple2() {
        let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_simple2.sgf"));

        let mut puzzle = Puzzle::<Profile>::new(tsumego);

        let won = puzzle.solve();

        assert!(won);
        // assert_eq!(puzzle.first_move(), Move::Place(BoardPosition::new(2, 1)));
        assert_display_snapshot!(puzzle.profiler.node_count, @"5453");
        assert_display_snapshot!(puzzle.profiler.max_depth, @"15");
    }

    #[test]
    fn true_simple3() {
        let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_simple3.sgf"));

        let mut puzzle = Puzzle::<Profile>::new(tsumego);

        let won = puzzle.solve();

        assert!(won);
        // assert_eq!(puzzle.first_move(), Move::Place(BoardPosition::new(5, 0)));
        assert_display_snapshot!(puzzle.profiler.node_count, @"219");
        assert_display_snapshot!(puzzle.profiler.max_depth, @"9");
    }

    #[test]
    fn true_simple4() {
        let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_simple4.sgf"));

        let mut puzzle = Puzzle::<Profile>::new(tsumego);

        let won = puzzle.solve();

        assert!(won);
        // assert_eq!(puzzle.first_move(), Move::Place(BoardPosition::new(7, 0)));
        assert_display_snapshot!(puzzle.profiler.node_count, @"59145");
        assert_display_snapshot!(puzzle.profiler.max_depth, @"18");
    }

    #[test]
    fn true_medium1() {
        let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_medium1.sgf"));

        let mut puzzle = Puzzle::<Profile>::new(tsumego);

        let won = puzzle.solve();

        assert!(won);
        // assert_eq!(puzzle.first_move(), Move::Place(BoardPosition::new(14, 2)));
        assert_display_snapshot!(puzzle.profiler.node_count, @"345725");
        assert_display_snapshot!(puzzle.profiler.max_depth, @"29");
    }

    #[test]
    fn true_ultrasimple1() {
        let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_ultrasimple1.sgf"));

        let mut puzzle = Puzzle::<Profile>::new(tsumego);

        let won = puzzle.solve();

        assert!(won);
        // assert_eq!(puzzle.first_move(), Move::Place(BoardPosition::new(1, 0)));
        assert_display_snapshot!(puzzle.profiler.node_count, @"5");
        assert_display_snapshot!(puzzle.profiler.max_depth, @"2");
    }

    #[test]
    fn true_ultrasimple2() {
        let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_ultrasimple2.sgf"));

        let mut puzzle = Puzzle::<Profile>::new(tsumego);

        let won = puzzle.solve();

        assert!(won);
        // assert_eq!(puzzle.first_move(), Move::Place(BoardPosition::new(1, 0)));
        assert_display_snapshot!(puzzle.profiler.node_count, @"173");
        assert_display_snapshot!(puzzle.profiler.max_depth, @"9");
    }

    #[test]
    fn trace_expanded_nodes() {
        let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_ultrasimple2.sgf"));
        let mut puzzle = Puzzle::<Profile>::new(tsumego);

        puzzle.solve();

        let mut output = String::new();
        let mut count = 1;
        for (node, depth) in puzzle.profiler.expanded_list {
            output.push_str(
                format!("{}, depth {}:\n{}\n\n", count, depth, node.get_board()).borrow(),
            );
            count += 1;
        }

        assert_snapshot!(output);
    }
}

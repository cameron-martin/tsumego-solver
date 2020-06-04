mod profiler;
mod solution;
mod terminal_detection;

use crate::go::{BoardPosition, GoGame, GoPlayer, Move};
pub use profiler::{NoProfile, Profile, Profiler};
use solution::Solution;
use std::{
    collections::HashSet,
    iter,
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

    pub fn solve(&mut self) -> Solution {
        self.solve_with_controller(&NoAbortController).unwrap()
    }

    pub fn solve_with_timeout(&mut self, timeout: Duration) -> Option<Solution> {
        self.solve_with_controller(&TimeoutAbortController::duration(timeout))
    }

    fn solve_with_controller<C: AbortController>(&mut self, controller: &C) -> Option<Solution> {
        let mut parents = HashSet::new();
        parents.insert(self.game);

        let mut max_depth: u8 = 1;

        loop {
            let mut variations = iter::repeat(Move::Place(BoardPosition::new(0, 0)))
                .take((max_depth as usize * (max_depth as usize + 1)) / 2)
                .collect();

            let result = self.negamax(
                self.game,
                -std::i8::MAX,
                std::i8::MAX,
                0,
                max_depth,
                1,
                controller,
                &mut parents,
                &mut variations,
                0,
            )?;

            if result != 0 {
                variations.truncate(max_depth as usize);

                return Some(Solution {
                    won: result > 0,
                    principle_variation: variations,
                });
            }

            max_depth += 1;
            self.profiler.move_down();
        }
    }

    fn negamax<C: AbortController>(
        &mut self,
        node: GoGame,
        alpha: i8,
        beta: i8,
        depth: u8,
        max_depth: u8,
        is_maximising_player: i8,
        controller: &C,
        parents: &mut HashSet<GoGame>,
        variations: &mut Vec<Move>,
        variations_index: usize,
    ) -> Option<i8> {
        if controller.should_abort() {
            return None;
        }

        self.profiler.visit_node();

        if let Some(value) = terminal_detection::is_terminal(node, self.player, self.attacker) {
            return Some(
                is_maximising_player
                    * if value {
                        std::i8::MAX - depth as i8
                    } else {
                        -(std::i8::MAX - depth as i8)
                    },
            );
        }

        if depth == max_depth {
            return Some(0);
        }

        let mut alpha = alpha;
        let this_variation_size = (max_depth - depth) as usize;
        let child_variation_size = this_variation_size - 1;
        let child_variations_index = variations_index + this_variation_size;

        let mut m = -std::i8::MAX;
        for (child, go_move) in node.generate_moves_including_pass() {
            if parents.contains(&child) {
                continue;
            }
            parents.insert(child);

            let t = -self.negamax(
                child,
                -beta,
                -alpha,
                depth + 1,
                max_depth,
                -is_maximising_player,
                controller,
                parents,
                variations,
                child_variations_index,
            )?;
            if t > m {
                m = t;
            }
            parents.remove(&child);
            if m >= beta {
                break;
            }

            if m > alpha {
                alpha = m;

                // Update principal variation
                variations[variations_index] = go_move;

                let (dst_arr, src_arr) = variations
                    [variations_index + 1..child_variations_index + child_variation_size]
                    .split_at_mut(child_variation_size);
                for (dst, src) in dst_arr.iter_mut().zip(src_arr) {
                    *dst = *src;
                }
            }
        }

        return Some(m);
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
    use insta::{assert_debug_snapshot, assert_display_snapshot, assert_snapshot};
    use profiler::Profile;
    use std::borrow::Borrow;

    fn show_principle_variation<T: Profiler>(puzzle: &Puzzle<T>, solution: &Solution) -> String {
        let games = solution
            .principle_variation
            .iter()
            .scan(puzzle.game, |state, &go_move| {
                *state = state.play_move(go_move).unwrap();

                Some((*state, go_move))
            });

        let mut output = String::new();
        output.push_str(format!("{}\n\n", puzzle.game.get_board()).borrow());
        for (game, go_move) in games {
            output.push_str(
                format!(
                    "{}: {}\n{}\n\n",
                    game.current_player.flip(),
                    go_move,
                    game.get_board()
                )
                .borrow(),
            );
        }

        output
    }

    #[test]
    fn true_simple1() {
        let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_simple1.sgf"));

        let mut puzzle = Puzzle::<Profile>::new(tsumego);

        let solution = puzzle.solve();

        assert!(solution.won);
        assert_display_snapshot!(puzzle.profiler.visited_nodes, @"644");
        assert_display_snapshot!(puzzle.profiler.max_depth, @"5");
        assert_snapshot!(show_principle_variation(&puzzle, &solution));
    }

    #[test]
    fn true_simple2() {
        let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_simple2.sgf"));

        let mut puzzle = Puzzle::<Profile>::new(tsumego);

        let solution = puzzle.solve();

        assert!(solution.won);
        assert_display_snapshot!(puzzle.profiler.visited_nodes, @"7295");
        assert_display_snapshot!(puzzle.profiler.max_depth, @"9");
        assert_snapshot!(show_principle_variation(&puzzle, &solution));
    }

    #[test]
    fn true_simple3() {
        let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_simple3.sgf"));

        let mut puzzle = Puzzle::<Profile>::new(tsumego);

        let solution = puzzle.solve();

        assert!(solution.won);
        assert_display_snapshot!(puzzle.profiler.visited_nodes, @"2946");
        assert_display_snapshot!(puzzle.profiler.max_depth, @"8");
        assert_snapshot!(show_principle_variation(&puzzle, &solution));
    }

    #[test]
    fn true_simple4() {
        let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_simple4.sgf"));

        let mut puzzle = Puzzle::<Profile>::new(tsumego);

        let solution = puzzle.solve();

        assert!(solution.won);
        assert_display_snapshot!(puzzle.profiler.visited_nodes, @"31143");
        assert_display_snapshot!(puzzle.profiler.max_depth, @"7");
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

        let mut puzzle = Puzzle::<Profile>::new(tsumego);

        let solution = puzzle.solve();

        assert!(solution.won);
        assert_display_snapshot!(puzzle.profiler.visited_nodes, @"5");
        assert_display_snapshot!(puzzle.profiler.max_depth, @"1");
        assert_snapshot!(show_principle_variation(&puzzle, &solution));
    }

    #[test]
    fn true_ultrasimple2() {
        let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_ultrasimple2.sgf"));

        let mut puzzle = Puzzle::<Profile>::new(tsumego);

        let solution = puzzle.solve();

        assert!(solution.won);
        assert_display_snapshot!(puzzle.profiler.visited_nodes, @"2170");
        assert_display_snapshot!(puzzle.profiler.max_depth, @"8");
        assert_snapshot!(show_principle_variation(&puzzle, &solution));
    }
}

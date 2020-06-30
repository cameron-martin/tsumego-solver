use super::{
    abort_controller::AbortController, example_collector::ExampleCollector,
    move_ranker::MoveRanker, solving_session::SolvingSession, terminal_detection, Profiler,
};
use crate::go::{BoardPosition, GoGame, Move};
use std::{iter, path::Path};

// Stores the state associated with an iteration of the iterative deepening algorithm
pub struct SolvingIteration<
    's,
    'e,
    C: AbortController,
    P: Profiler,
    E: ExampleCollector,
    R: MoveRanker,
> {
    max_depth: u8,
    session: &'s mut SolvingSession<'e, C, P, E, R>,
    pub variations: Vec<Move>,
}

impl<'a, 'b, C: AbortController, P: Profiler, E: ExampleCollector, R: MoveRanker>
    SolvingIteration<'a, 'b, C, P, E, R>
{
    pub fn new(max_depth: u8, session: &'a mut SolvingSession<'b, C, P, E, R>) -> Self {
        let variations = iter::repeat(Move::Place(BoardPosition::new(0, 0)))
            .take((max_depth as usize * (max_depth as usize + 1)) / 2)
            .collect();

        SolvingIteration {
            max_depth,
            variations,
            session,
        }
    }

    pub fn solve(&mut self) -> Option<i8> {
        let game = self.session.puzzle.game;
        self.session.parents.insert(game);
        let result = self.negamax(game, -1, 1, 0, 1, 0);
        self.session.parents.remove(&game);

        result
    }

    pub fn principle_variation(mut self) -> Vec<Move> {
        // self.variations.truncate(self.max_depth as usize);

        // self.variations

        Vec::new()
    }

    fn negamax(
        &mut self,
        node: GoGame,
        alpha: i8,
        beta: i8,
        depth: u8,
        is_maximising_player: i8,
        variations_index: usize,
    ) -> Option<i8> {
        if self.session.abort_controller.should_abort() {
            return None;
        }

        self.session.profiler.visit_node();

        if let Some(value) = terminal_detection::is_terminal(
            node,
            self.session.puzzle.player,
            self.session.puzzle.attacker,
        ) {
            return Some(is_maximising_player * if value { 1 } else { -1 });
        }

        if depth == self.max_depth {
            return Some(0);
        }

        let mut alpha = alpha;
        let this_variation_size = (self.max_depth - depth) as usize;
        let child_variation_size = this_variation_size - 1;
        let child_variations_index = variations_index + this_variation_size;

        let mut m = -1;
        // TODO: Make mode_dir a parameter
        for (i, (child, go_move)) in self.session.move_ranker.order_moves(node).enumerate() {
            if self.session.parents.contains(&child) {
                continue;
            }

            self.session.parents.insert(child);
            let t = -self.negamax(
                child,
                -beta,
                -alpha,
                depth + 1,
                -is_maximising_player,
                child_variations_index,
            )?;
            self.session.parents.remove(&child);
            if t > m {
                m = t;
            }
            if m >= beta {
                if m != 0 {
                    if i == 0 {
                        self.session.profiler.order_success();
                    } else {
                        self.session.profiler.order_miss();
                    }
                }
                break;
            }

            if m > alpha {
                alpha = m;

                // Update principal variation
                // self.variations[variations_index] = go_move;

                // let (dst_arr, src_arr) = self.variations
                //     [variations_index + 1..child_variations_index + child_variation_size]
                //     .split_at_mut(child_variation_size);
                // for (dst, src) in dst_arr.iter_mut().zip(src_arr) {
                //     *dst = *src;
                // }
            }
        }

        if m != 0 {
            self.session.example_collector.collect_example(
                node,
                if m > 0 {
                    node.current_player
                } else {
                    node.current_player.flip()
                },
            );
        }

        Some(m)
    }
}

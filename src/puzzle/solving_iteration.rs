use super::{
    abort_controller::AbortController, solving_session::SolvingSession, terminal_detection,
    Profiler,
};
use crate::go::{BoardPosition, GoGame, Move};
use std::iter;

// Stores the state associated with an iteration of the iterative deepening algorithm
pub struct SolvingIteration<'a, C: AbortController, P: Profiler> {
    max_depth: u8,
    session: &'a mut SolvingSession<C, P>,
    pub variations: Vec<Move>,
}

impl<'a, C: AbortController, P: Profiler> SolvingIteration<'a, C, P> {
    pub fn new(max_depth: u8, session: &'a mut SolvingSession<C, P>) -> Self {
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
        let result = self.negamax(game, -std::i8::MAX, std::i8::MAX, 0, 1, 0);
        self.session.parents.remove(&game);

        result
    }

    pub fn principle_variation(mut self) -> Vec<Move> {
        self.variations.truncate(self.max_depth as usize);

        self.variations
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
            return Some(
                is_maximising_player
                    * if value {
                        std::i8::MAX - depth as i8
                    } else {
                        -(std::i8::MAX - depth as i8)
                    },
            );
        }

        if depth == self.max_depth {
            return Some(0);
        }

        let mut alpha = alpha;
        let this_variation_size = (self.max_depth - depth) as usize;
        let child_variation_size = this_variation_size - 1;
        let child_variations_index = variations_index + this_variation_size;

        let mut m = -std::i8::MAX;
        for (child, go_move) in node.generate_moves_including_pass() {
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
                break;
            }

            if m > alpha {
                alpha = m;

                // Update principal variation
                self.variations[variations_index] = go_move;

                let (dst_arr, src_arr) = self.variations
                    [variations_index + 1..child_variations_index + child_variation_size]
                    .split_at_mut(child_variation_size);
                for (dst, src) in dst_arr.iter_mut().zip(src_arr) {
                    *dst = *src;
                }
            }
        }

        return Some(m);
    }
}

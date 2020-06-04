use super::{
    abort_controller::AbortController, solving_iteration::SolvingIteration, Profiler, Puzzle,
};
use crate::go::GoGame;
use std::collections::HashSet;

pub struct SolvingSession<C: AbortController, P: Profiler> {
    pub puzzle: Puzzle,
    pub parents: HashSet<GoGame>,
    pub profiler: P,
    pub abort_controller: C,
}

impl<C: AbortController, P: Profiler> SolvingSession<C, P> {
    pub fn new(puzzle: Puzzle, abort_controller: C) -> SolvingSession<C, P> {
        SolvingSession {
            parents: HashSet::new(),
            profiler: P::new(),
            puzzle,
            abort_controller,
        }
    }

    pub fn create_iteration(&mut self, max_depth: u8) -> SolvingIteration<C, P> {
        SolvingIteration::new(max_depth, self)
    }
}

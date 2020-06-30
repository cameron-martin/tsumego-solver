use super::{
    abort_controller::AbortController, example_collector::ExampleCollector,
    move_ranker::MoveRanker, solving_iteration::SolvingIteration, Profiler, Puzzle,
};
use crate::go::GoGame;
use std::{collections::HashSet, rc::Rc};

pub struct SolvingSession<'e, C: AbortController, P: Profiler, E: ExampleCollector, R: MoveRanker> {
    pub puzzle: Puzzle,
    pub move_ranker: Rc<R>,
    pub parents: HashSet<GoGame>,
    pub profiler: P,
    pub example_collector: &'e mut E,
    pub abort_controller: C,
}

impl<'e, C: AbortController, P: Profiler, E: ExampleCollector, R: MoveRanker>
    SolvingSession<'e, C, P, E, R>
{
    pub fn new(
        puzzle: Puzzle,
        abort_controller: C,
        example_collector: &'e mut E,
        move_ranker: Rc<R>,
    ) -> SolvingSession<C, P, E, R> {
        SolvingSession {
            parents: HashSet::new(),
            profiler: P::new(),
            example_collector,
            puzzle,
            abort_controller,
            move_ranker,
        }
    }

    pub fn create_iteration<'s>(
        &'s mut self,
        max_depth: u8,
    ) -> SolvingIteration<'s, 'e, C, P, E, R> {
        SolvingIteration::new(max_depth, self)
    }
}

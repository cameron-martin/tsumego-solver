use crate::go::Move;
use crate::puzzle::profiler::Profiler;

pub struct Solution<P: Profiler> {
    pub won: bool,
    pub principle_variation: Vec<Move>,
    pub profiler: P,
}

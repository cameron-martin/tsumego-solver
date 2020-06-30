use std::fmt;

pub trait Profiler {
    fn new() -> Self;
    fn move_up(&mut self);
    fn move_down(&mut self);
    fn visit_node(&mut self);
    fn order_success(&mut self);
    fn order_miss(&mut self);
}

pub struct NoProfile;

impl Profiler for NoProfile {
    fn new() -> NoProfile {
        NoProfile
    }

    fn move_up(&mut self) {}

    fn move_down(&mut self) {}

    fn visit_node(&mut self) {}

    fn order_success(&mut self) {}
    fn order_miss(&mut self) {}
}

pub struct Profile {
    current_depth: u8,
    pub max_depth: u8,
    pub visited_nodes: u32,
    successful_orderings: u32,
    missed_orderings: u32,
}

impl Profile {
    pub fn ordering_accuracy(&self) -> f32 {
        self.successful_orderings as f32
            / (self.successful_orderings + self.missed_orderings) as f32
    }
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Max Depth: {}\nVisited Nodes: {}\nOrdering Accuracy: {}\n",
            self.max_depth,
            self.visited_nodes,
            self.ordering_accuracy()
        )
    }
}

impl Profiler for Profile {
    fn new() -> Profile {
        Profile {
            current_depth: 1,
            max_depth: 1,
            visited_nodes: 0,
            successful_orderings: 0,
            missed_orderings: 0,
        }
    }

    fn move_up(&mut self) {
        self.current_depth -= 1;
    }

    fn move_down(&mut self) {
        self.current_depth += 1;
        if self.current_depth > self.max_depth {
            self.max_depth = self.current_depth;
        }
    }

    fn order_success(&mut self) {
        self.successful_orderings += 1;
    }

    fn order_miss(&mut self) {
        self.missed_orderings += 1;
    }

    fn visit_node(&mut self) {
        self.visited_nodes += 1;
    }
}

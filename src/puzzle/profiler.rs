pub trait Profiler {
    fn new() -> Self;
    fn move_up(&mut self);
    fn move_down(&mut self);
    fn visit_node(&mut self);
}

pub struct NoProfile;

impl Profiler for NoProfile {
    fn new() -> NoProfile {
        NoProfile
    }

    fn move_up(&mut self) {}

    fn move_down(&mut self) {}

    fn visit_node(&mut self) {}
}

pub struct Profile {
    current_depth: u8,
    pub max_depth: u8,
    pub visited_nodes: u32,
}

impl Profile {
    pub fn print(&self) -> String {
        format!(
            "Max Depth: {}\nVisited Nodes: {}\n",
            self.max_depth, self.visited_nodes
        )
    }
}

impl Profiler for Profile {
    fn new() -> Profile {
        Profile {
            current_depth: 1,
            max_depth: 1,
            visited_nodes: 0,
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

    fn visit_node(&mut self) {
        self.visited_nodes += 1;
    }
}

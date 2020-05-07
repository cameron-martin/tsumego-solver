pub trait Profiler {
    fn new() -> Self;
    fn move_up(&mut self);
    fn move_down(&mut self);
    fn add_nodes(&mut self, node_count: u8);
}

pub struct NoProfile;

impl Profiler for NoProfile {
    fn new() -> NoProfile {
        NoProfile
    }

    fn move_up(&mut self) {}

    fn move_down(&mut self) {}

    fn add_nodes(&mut self, _node_count: u8) {}
}

pub struct Profile {
    current_depth: u8,
    pub max_depth: u8,
    pub node_count: u32,
}

impl Profile {
    pub fn print(&self) -> String {
        format!(
            "Max Depth: {}\nNode Count: {}\n",
            self.max_depth, self.node_count
        )
    }
}

impl Profiler for Profile {
    fn new() -> Profile {
        Profile {
            current_depth: 1,
            max_depth: 1,
            node_count: 1,
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

    fn add_nodes(&mut self, node_count: u8) {
        self.node_count += node_count as u32;
    }
}

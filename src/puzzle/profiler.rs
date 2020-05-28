use crate::go::GoGame;
use std::collections::HashSet;

pub trait Profiler {
    fn new() -> Self;
    fn move_up(&mut self);
    fn move_down(&mut self);
    fn expand_node(&mut self, node: GoGame, child_count: u8);
}

pub struct NoProfile;

impl Profiler for NoProfile {
    fn new() -> NoProfile {
        NoProfile
    }

    fn move_up(&mut self) {}

    fn move_down(&mut self) {}

    fn expand_node(&mut self, _node: GoGame, _child_count: u8) {}
}

pub struct Profile {
    current_depth: u8,
    pub max_depth: u8,
    pub node_count: u32,
    pub expanded_list: Vec<(GoGame, u8)>,
    expanded_set: HashSet<GoGame>,
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
            current_depth: 0,
            max_depth: 0,
            node_count: 0,
            expanded_list: Vec::new(),
            expanded_set: HashSet::new(),
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

    fn expand_node(&mut self, node: GoGame, child_count: u8) {
        if !self.expanded_set.contains(&node) {
            self.expanded_set.insert(node);
            self.expanded_list.push((node, self.current_depth));
            self.node_count += child_count as u32;
        }
    }
}

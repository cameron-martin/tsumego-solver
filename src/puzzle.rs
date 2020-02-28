use crate::go::{GoGame, GoPlayer};
use petgraph::graph::NodeIndex;
use petgraph::{Direction, Graph};
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::ops::Add;

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub enum ProofNumber {
    Finite(u32),
    Infinity,
}

impl Debug for ProofNumber {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ProofNumber::Finite(n) => Debug::fmt(n, f),
            ProofNumber::Infinity => f.write_str("∞"),
        }
    }
}

impl Add for ProofNumber {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        match (self, rhs) {
            (ProofNumber::Finite(n), ProofNumber::Finite(m)) => ProofNumber::Finite(n + m),
            _ => ProofNumber::Infinity,
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum NodeValue {
    True,
    False,
    Unknown,
}

impl Debug for NodeValue {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(match self {
            NodeValue::False => "False",
            NodeValue::True => "True",
            NodeValue::Unknown => "Unknown",
        })
    }
}

pub enum NodeType {
    And,
    Or,
}

impl Debug for NodeType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(match self {
            NodeType::And => "And",
            NodeType::Or => "Or",
        })
    }
}

pub struct AndOrNode {
    of_type: NodeType,
    value: NodeValue,
    proof_number: ProofNumber,
    disproof_number: ProofNumber,
    game: GoGame,
}

impl Debug for AndOrNode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "{:?} {:?} {:?}/{:?}",
            self.of_type, self.value, self.proof_number, self.disproof_number
        ))
    }
}

impl AndOrNode {
    pub fn create_unknown_leaf(of_type: NodeType, game: GoGame) -> AndOrNode {
        AndOrNode {
            of_type,
            value: NodeValue::Unknown,
            proof_number: ProofNumber::Finite(1),
            disproof_number: ProofNumber::Finite(1),
            game,
        }
    }

    pub fn create_true_leaf(of_type: NodeType, game: GoGame) -> AndOrNode {
        AndOrNode {
            of_type,
            value: NodeValue::True,
            proof_number: ProofNumber::Finite(0),
            disproof_number: ProofNumber::Infinity,
            game,
        }
    }

    pub fn create_false_leaf(of_type: NodeType, game: GoGame) -> AndOrNode {
        AndOrNode {
            of_type,
            value: NodeValue::False,
            proof_number: ProofNumber::Infinity,
            disproof_number: ProofNumber::Finite(0),
            game,
        }
    }

    pub fn is_proved(&self) -> bool {
        self.proof_number == ProofNumber::Finite(0)
    }

    pub fn is_disproved(&self) -> bool {
        self.disproof_number == ProofNumber::Finite(0)
    }
}

pub struct Puzzle {
    player: GoPlayer,
    attacker: GoPlayer,
    tree: Graph<AndOrNode, ()>,
    root_id: NodeIndex,
}

impl Puzzle {
    pub fn new(game: GoGame, attacker: GoPlayer) -> Puzzle {
        // debug_assert_eq!(game.plys(), 0);

        let player = game.current_player;

        let mut tree = Graph::new();

        let root_id = tree.add_node(AndOrNode::create_unknown_leaf(NodeType::Or, game));

        Puzzle {
            player,
            attacker,
            tree,
            root_id,
        }
    }

    fn defender(&self) -> GoPlayer {
        self.attacker.flip()
    }

    fn develop_node(&mut self, node_id: NodeIndex) {
        debug_assert!(self.tree.neighbors(node_id).next().is_none());
        debug_assert_eq!(self.tree[node_id].value, NodeValue::Unknown);

        let mut any_false = false;
        let mut any_true = false;
        let mut any_unknown = false;

        let node = &self.tree[node_id];

        let mut moves = node.game.generate_moves();

        if !node.game.last_move_pass {
            moves.push(node.game.pass());
        }

        debug_assert_ne!(
            moves.len(),
            0,
            "{:?} {:?} to play\n{:?}",
            node,
            node.game.current_player,
            node.game.get_board()
        );

        for child in moves {
            let node = &self.tree[node_id];

            let node_type = if child.current_player == self.player {
                NodeType::Or
            } else {
                NodeType::And
            };

            let new_node = if !node
                .game
                .get_board()
                .unconditionally_alive_blocks_for_player(self.defender())
                .is_empty()
            {
                if self.defender() == self.player {
                    AndOrNode::create_true_leaf(node_type, child)
                } else {
                    AndOrNode::create_false_leaf(node_type, child)
                }
            } else if false {
                // TODO: Work out how we lose
                panic!()
            } else {
                AndOrNode::create_unknown_leaf(node_type, child)
            };

            match new_node.value {
                NodeValue::True => any_true = true,
                NodeValue::False => any_false = true,
                NodeValue::Unknown => any_unknown = true,
            }

            let new_node_id = self.tree.add_node(new_node);

            self.tree.add_edge(node_id, new_node_id, ());
        }

        let node = &mut self.tree[node_id];

        node.value = match node.of_type {
            NodeType::And => {
                if any_false {
                    NodeValue::False
                } else if any_unknown {
                    NodeValue::Unknown
                } else {
                    NodeValue::True
                }
            }
            NodeType::Or => {
                if any_true {
                    NodeValue::True
                } else if any_unknown {
                    NodeValue::Unknown
                } else {
                    NodeValue::False
                }
            }
        }
    }

    fn select_most_proving_node(&self, start_node_id: NodeIndex) -> NodeIndex {
        let mut current_node_id = start_node_id;

        loop {
            let node = &self.tree[current_node_id];

            let mut neighbours = self.tree.neighbors(current_node_id);

            if neighbours.clone().next().is_none() {
                debug_assert_eq!(
                    node.value,
                    NodeValue::Unknown,
                    "{:?} {:?} to play\n{:?}",
                    node,
                    node.game.current_player,
                    node.game.get_board()
                );
                break;
            }

            current_node_id = match node.of_type {
                NodeType::Or => {
                    debug_assert_ne!(
                        node.proof_number,
                        ProofNumber::Finite(0),
                        "{:?} {:?} to play\n{:?}",
                        node,
                        node.game.current_player,
                        node.game.get_board()
                    );

                    neighbours
                        .find(|&child_id| {
                            let child = &self.tree[child_id];

                            child.proof_number == node.proof_number
                        })
                        .unwrap()
                }
                NodeType::And => {
                    debug_assert_ne!(
                        node.disproof_number,
                        ProofNumber::Finite(0),
                        "{:?} {:?} to play\n{:?}",
                        node,
                        node.game.current_player,
                        node.game.get_board()
                    );

                    neighbours
                        .find(|&child_id| {
                            let child = &self.tree[child_id];

                            child.disproof_number == node.disproof_number
                        })
                        .unwrap()
                }
            }
        }

        current_node_id
    }

    pub fn solve(&mut self) {
        while !self.is_solved() {
            self.solve_iteration();
        }
    }

    fn is_solved(&self) -> bool {
        self.root_node().is_proved() || self.root_node().is_disproved()
    }

    fn solve_iteration(&mut self) {
        let most_proving_node_id = self.select_most_proving_node(self.root_id);

        self.develop_node(most_proving_node_id);
        self.update_ancestors(most_proving_node_id);
    }

    fn update_ancestors(&mut self, node_id: NodeIndex) {
        let mut current_node_id = node_id;

        loop {
            self.set_proof_and_disproof_numbers(current_node_id);

            if let Some(parent_node_id) = self
                .tree
                .neighbors_directed(current_node_id, Direction::Incoming)
                .next()
            {
                current_node_id = parent_node_id;
            } else {
                break;
            }
        }
    }

    fn set_proof_and_disproof_numbers(&mut self, node_id: NodeIndex) {
        let children = self
            .tree
            .neighbors(node_id)
            .map(|child_id| &self.tree[child_id]);

        let mut proof_number_sum = ProofNumber::Finite(0);
        let mut proof_number_min = ProofNumber::Infinity;
        let mut disproof_number_sum = ProofNumber::Finite(0);
        let mut disproof_number_min = ProofNumber::Infinity;

        for child in children {
            proof_number_sum = proof_number_sum + child.proof_number;
            disproof_number_sum = disproof_number_sum + child.disproof_number;

            if child.proof_number < proof_number_min {
                proof_number_min = child.proof_number;
            }

            if child.disproof_number < disproof_number_min {
                disproof_number_min = child.disproof_number;
            }
        }

        let node = &mut self.tree[node_id];
        match node.of_type {
            NodeType::And => {
                node.proof_number = proof_number_sum;
                node.disproof_number = disproof_number_min;
            }
            NodeType::Or => {
                node.proof_number = proof_number_min;
                node.disproof_number = disproof_number_sum;
            }
        }
    }

    fn root_node(&self) -> &AndOrNode {
        &self.tree[self.root_id]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::go::GoGame;

    #[test]
    fn true_simple1() {
        let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_simple1.sgf"));

        let mut puzzle = Puzzle::new(tsumego, GoPlayer::White);

        puzzle.solve();

        assert!(puzzle.root_node().is_proved());
    }

    #[test]
    fn true_simple2() {
        let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_simple2.sgf"));

        let mut puzzle = Puzzle::new(tsumego, GoPlayer::Black);

        puzzle.solve();

        assert!(puzzle.root_node().is_proved());
    }

    #[test]
    fn proof_number_ordering() {
        assert!(ProofNumber::Infinity > ProofNumber::Finite(1));
    }
}

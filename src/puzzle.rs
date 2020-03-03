use crate::go::{GoGame, GoPlayer, Move};
use petgraph::stable_graph::NodeIndex;
use petgraph::stable_graph::StableGraph;
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use std::cmp::Ordering;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::iter::Sum;
use std::ops::Add;

#[derive(PartialEq, Eq, Copy, Clone)]
pub struct ProofNumber(u32);

impl ProofNumber {
    fn infinite() -> ProofNumber {
        ProofNumber(0)
    }

    fn finite(n: u32) -> ProofNumber {
        ProofNumber(n + 1)
    }
}

impl PartialOrd for ProofNumber {
    fn partial_cmp(&self, other: &ProofNumber) -> Option<Ordering> {
        match (self.0, other.0) {
            (0, 0) => Some(Ordering::Equal),
            (0, _) => Some(Ordering::Greater),
            (_, 0) => Some(Ordering::Less),
            (n, m) => n.partial_cmp(&m),
        }
    }
}

impl Ord for ProofNumber {
    fn cmp(&self, other: &ProofNumber) -> Ordering {
        match (self.0, other.0) {
            (0, 0) => Ordering::Equal,
            (0, _) => Ordering::Greater,
            (_, 0) => Ordering::Less,
            (n, m) => n.cmp(&m),
        }
    }
}

impl Sum for ProofNumber {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut sum = 1;

        for n in iter {
            match n.0 {
                0 => return ProofNumber(0),
                m => sum += m - 1,
            }
        }

        ProofNumber(sum)
    }
}

impl Debug for ProofNumber {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self.0 {
            0 => f.write_str("∞"),
            n => Debug::fmt(&(n - 1), f),
        }
    }
}

impl Add for ProofNumber {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        match (self.0, rhs.0) {
            (0, _) => ProofNumber(0),
            (_, 0) => ProofNumber(0),
            (n, m) => ProofNumber((n - 1) + m),
        }
    }
}

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
pub struct AndOrNode {
    of_type: NodeType,
    proof_number: ProofNumber,
    disproof_number: ProofNumber,
    game: GoGame,
}

impl Debug for AndOrNode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "Type: {:?}, Proof/Disproof Numbers: {:?}/{:?}, To Play: {:?}, Board:\n{:?}",
            self.of_type,
            self.proof_number,
            self.disproof_number,
            self.game.current_player,
            self.game.get_board()
        ))
    }
}

impl AndOrNode {
    pub fn create_non_terminal_leaf(of_type: NodeType, game: GoGame) -> AndOrNode {
        AndOrNode {
            of_type,
            proof_number: ProofNumber::finite(1),
            disproof_number: ProofNumber::finite(1),
            game,
        }
    }

    pub fn create_true_leaf(of_type: NodeType, game: GoGame) -> AndOrNode {
        AndOrNode {
            of_type,
            proof_number: ProofNumber::finite(0),
            disproof_number: ProofNumber::infinite(),
            game,
        }
    }

    pub fn create_false_leaf(of_type: NodeType, game: GoGame) -> AndOrNode {
        AndOrNode {
            of_type,
            proof_number: ProofNumber::infinite(),
            disproof_number: ProofNumber::finite(0),
            game,
        }
    }

    pub fn is_proved(&self) -> bool {
        self.proof_number == ProofNumber::finite(0)
    }

    pub fn is_disproved(&self) -> bool {
        self.disproof_number == ProofNumber::finite(0)
    }

    pub fn is_solved(&self) -> bool {
        self.is_proved() || self.is_disproved()
    }
}

pub struct Puzzle {
    player: GoPlayer,
    attacker: GoPlayer,
    pub tree: StableGraph<AndOrNode, Move>,
    pub root_id: NodeIndex,
}

impl Puzzle {
    pub fn new(game: GoGame, attacker: GoPlayer) -> Puzzle {
        // debug_assert_eq!(game.plys(), 0);

        let player = game.current_player;

        let mut tree = StableGraph::<AndOrNode, Move>::new();

        let root_id = tree.add_node(AndOrNode::create_non_terminal_leaf(NodeType::Or, game));

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

        let node = self.tree[node_id];

        let mut moves = node.game.generate_moves();

        let node_type = if node.game.current_player == self.player {
            NodeType::And
        } else {
            NodeType::Or
        };

        let mut not_empty = false;

        if !node.game.last_move_pass {
            moves.push((node.game.pass(), Move::PassOnce));
        } else {
            let new_node_id = self
                .tree
                .add_node(if node.game.current_player == self.player {
                    AndOrNode::create_false_leaf(node_type, node.game.pass())
                } else {
                    AndOrNode::create_true_leaf(node_type, node.game.pass())
                });

            self.tree.add_edge(node_id, new_node_id, Move::PassTwice);

            not_empty = true;
        }

        debug_assert!(
            moves.len() != 0 || not_empty,
            "No moves found for node: {:?}",
            node
        );

        for (child, board_move) in moves {
            let new_node = if !child
                .get_board()
                .unconditionally_alive_blocks_for_player(self.defender())
                .is_empty()
            {
                if self.defender() == self.player {
                    AndOrNode::create_true_leaf(node_type, child)
                } else {
                    AndOrNode::create_false_leaf(node_type, child)
                }
            } else if self.is_defender_dead(child) {
                if self.attacker == self.player {
                    AndOrNode::create_true_leaf(node_type, child)
                } else {
                    AndOrNode::create_false_leaf(node_type, child)
                }
            } else {
                AndOrNode::create_non_terminal_leaf(node_type, child)
            };

            let new_node_id = self.tree.add_node(new_node);

            self.tree.add_edge(node_id, new_node_id, board_move);
        }
    }

    /// A conservative estimate on whether the group is dead.
    /// true means it's definitely dead, false otherwise
    fn is_defender_dead(&self, game: GoGame) -> bool {
        let attacker_alive = game
            .out_of_bounds
            .expand_one()
            .flood_fill(game.get_board().get_bitboard_for_player(self.attacker));

        let maximum_living_shape = !attacker_alive & !game.out_of_bounds;

        maximum_living_shape.interior().count() < 2
    }

    fn select_most_proving_node(&self, start_node_id: NodeIndex) -> NodeIndex {
        let mut current_node_id = start_node_id;

        loop {
            let node = self.tree[current_node_id];

            let mut neighbours = self.tree.neighbors(current_node_id);

            if self.tree.neighbors(current_node_id).next().is_none() {
                break;
            }

            current_node_id = match node.of_type {
                NodeType::Or => {
                    debug_assert_ne!(node.proof_number, ProofNumber::finite(0), "{:?}", node);

                    neighbours
                        .find(|&child_id| {
                            let child = self.tree[child_id];

                            child.proof_number == node.proof_number
                        })
                        .unwrap()
                }
                NodeType::And => {
                    debug_assert_ne!(node.disproof_number, ProofNumber::finite(0), "{:?}", node);

                    neighbours
                        .find(|&child_id| {
                            let child = self.tree[child_id];

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
        self.root_node().is_solved()
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
            self.prune_if_solved(current_node_id);

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
            .map(|child_id| self.tree[child_id]);

        let mut proof_number_sum = ProofNumber::finite(0);
        let mut proof_number_min = ProofNumber::infinite();
        let mut disproof_number_sum = ProofNumber::finite(0);
        let mut disproof_number_min = ProofNumber::infinite();

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

    fn prune_if_solved(&mut self, node_id: NodeIndex) {
        // Don't prune the root
        if node_id == self.root_id {
            return;
        }

        let node = self.tree[node_id];

        if node.is_solved() {
            let mut walker = self.tree.neighbors(node_id).detach();
            while let Some(child_id) = walker.next_node(&self.tree) {
                self.tree.remove_node(child_id);
            }
        }
    }

    fn root_node(&self) -> AndOrNode {
        self.tree[self.root_id]
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

        assert!(puzzle.root_node().is_proved(), "{:?}", puzzle.root_node());
    }

    #[test]
    fn true_simple3() {
        let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_simple3.sgf"));

        let mut puzzle = Puzzle::new(tsumego, GoPlayer::Black);

        puzzle.solve();

        assert!(puzzle.root_node().is_proved(), "{:?}", puzzle.root_node());
    }

    #[test]
    fn proof_number_ordering() {
        assert!(ProofNumber::infinite() > ProofNumber::finite(1));
    }
}

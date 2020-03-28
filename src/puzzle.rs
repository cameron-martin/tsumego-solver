mod proof_number;

use crate::go::{GoBoard, GoGame, GoPlayer, Move};
use petgraph::stable_graph::NodeIndex;
use petgraph::stable_graph::StableGraph;
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use proof_number::ProofNumber;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::time::Duration;
use std::time::Instant;

#[derive(Clone, Copy)]
pub enum NodeType {
    And,
    Or,
}

impl NodeType {
    fn flip(self) -> NodeType {
        match self {
            NodeType::And => NodeType::Or,
            NodeType::Or => NodeType::And,
        }
    }
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
    proof_number: ProofNumber,
    disproof_number: ProofNumber,
}

impl Debug for AndOrNode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "Proof/Disproof Numbers: {:?}/{:?}",
            // self.of_type,
            self.proof_number,
            self.disproof_number,
            // self.game.current_player,
            // self.game.get_board()
        ))
    }
}

impl AndOrNode {
    pub fn create_non_terminal_leaf() -> AndOrNode {
        AndOrNode {
            proof_number: ProofNumber::finite(1),
            disproof_number: ProofNumber::finite(1),
        }
    }

    pub fn create_true_leaf() -> AndOrNode {
        AndOrNode {
            proof_number: ProofNumber::finite(0),
            disproof_number: ProofNumber::infinite(),
        }
    }

    pub fn create_false_leaf() -> AndOrNode {
        AndOrNode {
            proof_number: ProofNumber::infinite(),
            disproof_number: ProofNumber::finite(0),
        }
    }

    pub fn is_proved(self) -> bool {
        self.proof_number == ProofNumber::finite(0)
    }

    pub fn is_disproved(self) -> bool {
        self.disproof_number == ProofNumber::finite(0)
    }

    pub fn is_solved(self) -> bool {
        self.is_proved() || self.is_disproved()
    }
}

pub struct Puzzle {
    player: GoPlayer,
    attacker: GoPlayer,
    pub tree: StableGraph<AndOrNode, Move>,
    pub root_id: NodeIndex,
    current_node_id: NodeIndex,
    game_stack: Vec<GoGame>,
    current_type: NodeType,
}

impl Puzzle {
    pub fn new(game: GoGame) -> Puzzle {
        // debug_assert_eq!(game.plys(), 0);

        let attacker = if !(game.get_board().out_of_bounds().expand_one()
            & game.get_board().get_bitboard_for_player(GoPlayer::White))
        .is_empty()
        {
            GoPlayer::White
        } else {
            GoPlayer::Black
        };

        let player = game.current_player;

        let mut tree = StableGraph::<AndOrNode, Move>::new();

        let root_id = tree.add_node(AndOrNode::create_non_terminal_leaf());

        Puzzle {
            player,
            attacker,
            tree,
            root_id,
            current_node_id: root_id,
            game_stack: vec![game],
            current_type: NodeType::Or,
        }
    }

    pub fn from_sgf(sgf_string: &str) -> Puzzle {
        Self::new(GoGame::from_sgf(sgf_string))
    }

    fn defender(&self) -> GoPlayer {
        self.attacker.flip()
    }

    fn current_game(&self) -> GoGame {
        *self.game_stack.last().unwrap()
    }

    fn develop_current_node(&mut self) {
        debug_assert!(self.tree.neighbors(self.current_node_id).next().is_none());

        let game = self.current_game();

        let mut moves = game.generate_moves();

        let mut not_empty = false;

        if !game.last_move_pass {
            moves.push((game.pass(), Move::PassOnce));
        } else {
            let new_node_id = self.tree.add_node(if game.current_player == self.player {
                AndOrNode::create_false_leaf()
            } else {
                AndOrNode::create_true_leaf()
            });

            self.tree
                .add_edge(self.current_node_id, new_node_id, Move::PassTwice);

            not_empty = true;
        }

        debug_assert!(
            !moves.is_empty() || not_empty,
            "No moves found for node: {:?}",
            game
        );

        for (child, board_move) in moves {
            let new_node = if !child
                .get_board()
                .unconditionally_alive_blocks_for_player(self.defender())
                .is_empty()
            {
                if self.defender() == self.player {
                    AndOrNode::create_true_leaf()
                } else {
                    AndOrNode::create_false_leaf()
                }
            } else if self.is_defender_dead(child.get_board()) {
                if self.attacker == self.player {
                    AndOrNode::create_true_leaf()
                } else {
                    AndOrNode::create_false_leaf()
                }
            } else {
                AndOrNode::create_non_terminal_leaf()
            };

            let new_node_id = self.tree.add_node(new_node);

            self.tree
                .add_edge(self.current_node_id, new_node_id, board_move);
        }
    }

    /// A conservative estimate on whether the group is dead.
    /// true means it's definitely dead, false otherwise
    fn is_defender_dead(&self, board: GoBoard) -> bool {
        let attacker_alive = board
            .out_of_bounds()
            .expand_one()
            .flood_fill(board.get_bitboard_for_player(self.attacker));

        let maximum_living_shape = !attacker_alive & !board.out_of_bounds();

        maximum_living_shape.interior().count() < 2
    }

    fn select_most_proving_node(&mut self) {
        loop {
            let mut outgoing_edges = self.tree.edges(self.current_node_id);

            if self.tree.neighbors(self.current_node_id).next().is_none() {
                break;
            }

            let node = self.tree[self.current_node_id];
            let chosen_edge = match self.current_type {
                NodeType::Or => {
                    debug_assert_ne!(node.proof_number, ProofNumber::finite(0), "{:?}", node);

                    outgoing_edges
                        .find(|&edge_ref| {
                            let child = self.tree[edge_ref.target()];

                            child.proof_number == node.proof_number
                        })
                        .unwrap()
                }
                NodeType::And => {
                    debug_assert_ne!(node.disproof_number, ProofNumber::finite(0), "{:?}", node);

                    outgoing_edges
                        .find(|&edge_ref| {
                            let child = self.tree[edge_ref.target()];

                            child.disproof_number == node.disproof_number
                        })
                        .unwrap()
                }
            };

            self.current_node_id = chosen_edge.target();
            self.current_type = self.current_type.flip();
            self.game_stack.push(
                self.current_game()
                    .play_move(*chosen_edge.weight())
                    .unwrap(),
            );
        }
    }

    pub fn solve(&mut self) {
        while !self.is_solved() {
            self.solve_iteration();
        }
    }

    pub fn solve_with_timeout(&mut self, timeout: Duration) -> bool {
        let timeout_at = Instant::now() + timeout;

        while !self.is_solved() {
            if Instant::now() > timeout_at {
                return false;
            }

            self.solve_iteration();
        }

        true
    }

    pub fn is_solved(&self) -> bool {
        self.root_node().is_solved()
    }

    pub fn is_proved(&self) -> bool {
        self.root_node().is_proved()
    }

    fn solve_iteration(&mut self) {
        self.select_most_proving_node();
        self.develop_current_node();
        self.update_ancestors();
    }

    fn update_ancestors(&mut self) {
        loop {
            let has_changed = self.set_proof_and_disproof_numbers();

            if !has_changed {
                break;
            }

            self.prune_if_solved();

            if let Some(parent_node_id) = self
                .tree
                .neighbors_directed(self.current_node_id, Direction::Incoming)
                .next()
            {
                self.current_node_id = parent_node_id;
                self.current_type = self.current_type.flip();
                self.game_stack.pop();
            } else {
                break;
            }
        }
    }

    fn set_proof_and_disproof_numbers(&mut self) -> bool {
        let children = self
            .tree
            .neighbors(self.current_node_id)
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

        let node = &mut self.tree[self.current_node_id];
        match self.current_type {
            NodeType::And => {
                let has_changed = proof_number_sum != node.proof_number
                    || disproof_number_min != node.disproof_number;

                node.proof_number = proof_number_sum;
                node.disproof_number = disproof_number_min;

                has_changed
            }
            NodeType::Or => {
                let has_changed = proof_number_min != node.proof_number
                    || disproof_number_sum != node.disproof_number;

                node.proof_number = proof_number_min;
                node.disproof_number = disproof_number_sum;

                has_changed
            }
        }
    }

    fn prune_if_solved(&mut self) {
        // Don't prune the root
        if self.current_node_id == self.root_id {
            return;
        }

        let node = self.tree[self.current_node_id];

        if node.is_solved() {
            let mut walker = self.tree.neighbors(self.current_node_id).detach();
            while let Some(child_id) = walker.next_node(&self.tree) {
                self.tree.remove_node(child_id);
            }
        }
    }

    fn root_node(&self) -> AndOrNode {
        self.tree[self.root_id]
    }

    pub fn first_move(&self) -> Move {
        *self
            .tree
            .edges(self.root_id)
            .find(|edge| self.tree[edge.target()].is_proved())
            .unwrap()
            .weight()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::go::{BoardPosition, GoGame};

    #[test]
    fn true_simple1() {
        let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_simple1.sgf"));

        let mut puzzle = Puzzle::new(tsumego);

        puzzle.solve();

        assert!(puzzle.root_node().is_proved());
        assert_eq!(puzzle.first_move(), Move::Place(BoardPosition::new(4, 0)));
    }

    #[test]
    fn true_simple2() {
        let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_simple2.sgf"));

        let mut puzzle = Puzzle::new(tsumego);

        puzzle.solve();

        assert!(puzzle.root_node().is_proved(), "{:?}", puzzle.root_node());
        assert_eq!(puzzle.first_move(), Move::Place(BoardPosition::new(2, 1)));
    }

    #[test]
    fn true_simple3() {
        let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_simple3.sgf"));

        let mut puzzle = Puzzle::new(tsumego);

        puzzle.solve();

        assert!(puzzle.root_node().is_proved(), "{:?}", puzzle.root_node());
        assert_eq!(puzzle.first_move(), Move::Place(BoardPosition::new(5, 0)));
    }

    #[test]
    fn true_simple4() {
        let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_simple4.sgf"));

        let mut puzzle = Puzzle::new(tsumego);

        puzzle.solve();

        assert!(puzzle.root_node().is_proved(), "{:?}", puzzle.root_node());
        assert_eq!(puzzle.first_move(), Move::Place(BoardPosition::new(7, 0)));
    }

    #[test]
    fn true_medium1() {
        let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_medium1.sgf"));

        let mut puzzle = Puzzle::new(tsumego);

        puzzle.solve();

        assert!(puzzle.root_node().is_proved(), "{:?}", puzzle.root_node());
        assert_eq!(puzzle.first_move(), Move::Place(BoardPosition::new(14, 2)));
    }
}

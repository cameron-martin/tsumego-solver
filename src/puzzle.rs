mod profiler;
mod proof_number;

use crate::go::{GoBoard, GoGame, GoPlayer, Move, PassState};
pub use profiler::{NoProfile, Profile, Profiler};
use proof_number::ProofNumber;
use std::cmp;
use std::collections::HashMap;
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
    phi: ProofNumber,
    delta: ProofNumber,
}

impl Debug for AndOrNode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "Phi/Delta: {:?}/{:?}",
            // self.of_type,
            self.phi,
            self.delta,
            // self.game.current_player,
            // self.game.get_board()
        ))
    }
}

impl AndOrNode {
    pub fn create_non_terminal_leaf() -> AndOrNode {
        AndOrNode {
            phi: ProofNumber::finite(1),
            delta: ProofNumber::finite(1),
        }
    }
}

struct Node {
    game: GoGame,
    phi_threshold: ProofNumber,
    delta_threshold: ProofNumber,
    children: Vec<GoGame>,
}

pub struct Puzzle<P: Profiler> {
    root_game: GoGame,
    tt: HashMap<GoGame, AndOrNode>,
    player: GoPlayer,
    attacker: GoPlayer,
    profiler: P,
}

impl<P: Profiler> Puzzle<P> {
    pub fn new(game: GoGame) -> Puzzle<P> {
        let attacker = if !(game.get_board().out_of_bounds().expand_one()
            & game.get_board().get_bitboard_for_player(GoPlayer::White))
        .is_empty()
        {
            GoPlayer::White
        } else {
            GoPlayer::Black
        };
        Puzzle {
            root_game: game,
            tt: HashMap::new(),
            player: game.current_player,
            attacker,
            profiler: P::new(),
        }
    }

    pub fn from_sgf(sgf_string: &str) -> Puzzle<P> {
        Self::new(GoGame::from_sgf(sgf_string))
    }

    fn defender(&self) -> GoPlayer {
        self.attacker.flip()
    }

    fn is_terminal(&self, game: GoGame) -> Option<bool> {
        // If both players pass sequentially, the game ends and
        // the player to pass second loses.
        if game.pass_state == PassState::PassedTwice {
            Some(game.current_player == self.player)
        // If the defender has unconditionally alive blocks, the defender wins
        } else if !game
            .get_board()
            .unconditionally_alive_blocks_for_player(self.defender())
            .is_empty()
        {
            Some(self.defender() == self.player)
        // If the defender doesn't have any space to create eyes, the attacker wins
        } else if self.is_defender_dead(game.get_board()) {
            Some(self.attacker == self.player)
        // Otherwise, the result is a non-terminal node
        } else {
            None
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

    pub fn solve(&mut self) -> bool {
        let mut root_node = Node {
            game: self.root_game,
            phi_threshold: ProofNumber::infinite(),
            delta_threshold: ProofNumber::infinite(),
            children: Vec::new(),
        };

        self.mid_profile(&mut root_node);

        root_node.delta_threshold == ProofNumber::infinite()
    }

    fn mid_profile(&mut self, node: &mut Node) {
        self.profiler.move_down();
        self.mid(node);
        self.profiler.move_up();
    }

    fn mid(&mut self, node: &mut Node) {
        let proof_numbers = self.tt_get(node.game);

        if proof_numbers.phi >= node.phi_threshold || proof_numbers.delta >= node.delta_threshold {
            node.delta_threshold = proof_numbers.delta;
            node.phi_threshold = proof_numbers.phi;
            return;
        }

        if let Some(game_theoretic_value) = self.is_terminal(node.game) {
            if self.is_or_node(node.game) ^ game_theoretic_value {
                node.phi_threshold = ProofNumber::infinite();
                node.delta_threshold = ProofNumber::finite(0);
            } else {
                node.phi_threshold = ProofNumber::finite(0);
                node.delta_threshold = ProofNumber::infinite();
            }

            self.tt_store_thresholds(node);
            return;
        }

        // TODO: Create a "caterpillar tree stack" to store these child relationships
        for (child, _) in node.game.generate_moves_including_pass() {
            node.children.push(child);
        }

        self.profiler
            .expand_node(node.game, node.children.len() as u8);

        self.tt_store_thresholds(node);

        loop {
            let min = self.min(node);
            let sum = self.sum(node);
            if node.phi_threshold <= min || node.delta_threshold <= sum {
                node.phi_threshold = min;
                node.delta_threshold = sum;
                self.tt_store_thresholds(node);
                return;
            }

            let (n_c, phi_c, delta_2) = self.select_child(node, proof_numbers.phi);

            let mut child_node = Node {
                game: n_c,
                phi_threshold: node.delta_threshold + phi_c - sum,
                delta_threshold: cmp::min(node.phi_threshold, delta_2 + ProofNumber::finite(1)),
                children: Vec::new(),
            };

            self.mid_profile(&mut child_node);
        }
    }

    fn select_child(&self, n: &Node, delta_c: ProofNumber) -> (GoGame, ProofNumber, ProofNumber) {
        let delta_bound = delta_c;

        let mut delta_c = ProofNumber::infinite();
        let mut delta_2 = ProofNumber::infinite();
        let mut phi_c = ProofNumber::infinite();

        let mut n_best = None;

        for &child in n.children.iter() {
            let AndOrNode { phi, mut delta } = self.tt_get(child);

            if phi != ProofNumber::infinite() {
                delta = cmp::max(delta, delta_bound);
            }
            if delta < delta_c {
                n_best = Some(child);
                delta_2 = delta_c;
                phi_c = phi;
                delta_c = delta;
            } else if delta < delta_2 {
                delta_2 = delta;
            }

            if phi == ProofNumber::infinite() {
                break;
            }
        }

        (n_best.unwrap(), phi_c, delta_2)
    }

    fn min(&self, node: &Node) -> ProofNumber {
        node.children
            .iter()
            .map(|&child| self.tt_get(child).delta)
            .min()
            .unwrap()
    }

    fn sum(&self, node: &Node) -> ProofNumber {
        node.children
            .iter()
            .map(|&child| self.tt_get(child).phi)
            .sum()
    }

    fn is_and_node(&self, game: GoGame) -> bool {
        !self.is_or_node(game)
    }

    fn is_or_node(&self, game: GoGame) -> bool {
        game.current_player == self.player
    }

    fn tt_store_thresholds(&mut self, node: &Node) {
        self.tt_store(
            node.game,
            AndOrNode {
                phi: node.phi_threshold,
                delta: node.delta_threshold,
            },
        )
    }

    fn tt_store(&mut self, game: GoGame, value: AndOrNode) {
        self.tt.insert(game, value);
    }

    fn tt_get(&self, game: GoGame) -> AndOrNode {
        *self
            .tt
            .get(&game)
            .unwrap_or(&AndOrNode::create_non_terminal_leaf())
    }

    pub fn first_move(&self) -> Move {
        let children = self.root_game.generate_moves_including_pass();

        let (_, board_move) = children
            .iter()
            .find(|(child, _)| {
                let AndOrNode { phi, delta: _ } = self.tt_get(*child);

                phi == ProofNumber::infinite()
            })
            .unwrap();

        *board_move
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::go::{BoardPosition, GoGame};
    use insta::{assert_display_snapshot, assert_snapshot};
    use profiler::Profile;
    use std::borrow::Borrow;

    #[test]
    fn true_simple1() {
        let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_simple1.sgf"));

        let mut puzzle = Puzzle::<Profile>::new(tsumego);

        let won = puzzle.solve();

        assert!(won);
        assert_eq!(puzzle.first_move(), Move::Place(BoardPosition::new(4, 0)));
        assert_display_snapshot!(puzzle.profiler.node_count, @"1146");
        assert_display_snapshot!(puzzle.profiler.max_depth, @"6");
    }

    #[test]
    fn true_simple2() {
        let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_simple2.sgf"));

        let mut puzzle = Puzzle::<Profile>::new(tsumego);

        let won = puzzle.solve();

        assert!(won);
        assert_eq!(puzzle.first_move(), Move::Place(BoardPosition::new(2, 1)));
        assert_display_snapshot!(puzzle.profiler.node_count, @"10500");
        assert_display_snapshot!(puzzle.profiler.max_depth, @"14");
    }

    #[test]
    fn true_simple3() {
        let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_simple3.sgf"));

        let mut puzzle = Puzzle::<Profile>::new(tsumego);

        let won = puzzle.solve();

        assert!(won);
        assert_eq!(puzzle.first_move(), Move::Place(BoardPosition::new(5, 0)));
        assert_display_snapshot!(puzzle.profiler.node_count, @"313");
        assert_display_snapshot!(puzzle.profiler.max_depth, @"9");
    }

    #[test]
    fn true_simple4() {
        let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_simple4.sgf"));

        let mut puzzle = Puzzle::<Profile>::new(tsumego);

        let won = puzzle.solve();

        assert!(won);
        assert_eq!(puzzle.first_move(), Move::Place(BoardPosition::new(7, 0)));
        assert_display_snapshot!(puzzle.profiler.node_count, @"537429");
        assert_display_snapshot!(puzzle.profiler.max_depth, @"12");
    }

    #[test]
    fn true_medium1() {
        let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_medium1.sgf"));

        let mut puzzle = Puzzle::<Profile>::new(tsumego);

        let won = puzzle.solve();

        assert!(won);
        assert_eq!(puzzle.first_move(), Move::Place(BoardPosition::new(14, 2)));
        assert_display_snapshot!(puzzle.profiler.node_count, @"409993");
        assert_display_snapshot!(puzzle.profiler.max_depth, @"28");
    }

    #[test]
    fn true_ultrasimple1() {
        let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_ultrasimple1.sgf"));

        let mut puzzle = Puzzle::<Profile>::new(tsumego);

        let won = puzzle.solve();

        assert!(won);
        assert_eq!(puzzle.first_move(), Move::Place(BoardPosition::new(1, 0)));
        assert_display_snapshot!(puzzle.profiler.node_count, @"7");
        assert_display_snapshot!(puzzle.profiler.max_depth, @"2");
    }

    #[test]
    fn true_ultrasimple2() {
        let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_ultrasimple2.sgf"));

        let mut puzzle = Puzzle::<Profile>::new(tsumego);

        let won = puzzle.solve();

        assert!(won);
        assert_eq!(puzzle.first_move(), Move::Place(BoardPosition::new(1, 0)));
        assert_display_snapshot!(puzzle.profiler.node_count, @"237");
        assert_display_snapshot!(puzzle.profiler.max_depth, @"9");
    }

    #[test]
    fn trace_expanded_nodes() {
        let tsumego = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_ultrasimple2.sgf"));
        let mut puzzle = Puzzle::<Profile>::new(tsumego);

        puzzle.solve();

        let mut output = String::new();
        let mut count = 1;
        for (node, depth) in puzzle.profiler.expanded_list {
            output.push_str(format!("{}, depth {}:\n{}\n\n", count, depth, node.get_board()).borrow());
            count += 1;
        }

        assert_snapshot!(output);
    }
}

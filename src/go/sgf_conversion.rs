use super::{BitBoard, BoardCell, BoardPosition, GoBoard, GoGame, GoPlayer, Move};
use sgf_parser;
use sgf_parser::{Action, Color, GameNode, GameTree, SgfToken};

impl From<Color> for GoPlayer {
    fn from(color: Color) -> Self {
        match color {
            Color::Black => GoPlayer::Black,
            Color::White => GoPlayer::White,
        }
    }
}

impl Into<Color> for GoPlayer {
    fn into(self) -> Color {
        match self {
            GoPlayer::Black => Color::Black,
            GoPlayer::White => Color::White,
        }
    }
}

impl GoGame {
    pub fn from_sgf(sgf_string: &str) -> GoGame {
        let sgf = sgf_parser::parse(sgf_string).unwrap();

        assert_eq!(sgf.count_variations(), 0);

        let mut nodes = sgf.iter();

        let first_node = nodes.next().unwrap();

        let mut board = GoBoard::empty();
        let mut triangle_location = None;

        for token in first_node.tokens.iter() {
            match token {
                SgfToken::Add {
                    color,
                    coordinate: (i, j),
                } => board.set_cell(
                    BoardPosition::new(i - 1, j - 1),
                    BoardCell::Occupied((*color).into()),
                ),
                SgfToken::Triangle { coordinate: (i, j) } => {
                    triangle_location = Some(BoardPosition::new(i - 1, j - 1))
                }
                SgfToken::Move { .. } => panic!("Cannot move at this time!"),
                _ => {}
            }
        }

        if let Some(position) = triangle_location {
            board.set_out_of_bounds(BitBoard::singleton(position).flood_fill(board.empty_cells()));
        };

        let mut game = GoGame::from_board(board, GoPlayer::Black);

        for node in nodes {
            for token in node.tokens.iter() {
                match token {
                    SgfToken::Move {
                        color,
                        action: Action::Move(i, j),
                    } => {
                        game = game
                            .play_move_for_player(
                                Move::Place(BoardPosition::new(i - 1, j - 1)),
                                (*color).into(),
                            )
                            .unwrap()
                    }
                    SgfToken::Add { .. } => panic!("Cannot add stones at this time!"),
                    _ => {}
                }
            }
        }

        game
    }
}

impl GoBoard {
    pub fn to_sgf(&self) -> String {
        let mut tokens: Vec<_> = GoPlayer::both()
            .flat_map(|&go_player| {
                let board = self.get_bitboard_for_player(go_player);

                board.positions().map(move |position| {
                    let (x, y) = position.to_pair();

                    SgfToken::Add {
                        color: go_player.into(),
                        coordinate: (x + 1, y + 1),
                    }
                })
            })
            .collect();

        tokens.push(SgfToken::Triangle {
            coordinate: {
                let (x, y) = self.out_of_bounds().positions().next().unwrap().to_pair();

                (x + 1, y + 1)
            },
        });

        let node = GameNode { tokens };

        let tree = GameTree {
            nodes: vec![node],
            variations: Vec::new(),
        };

        tree.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation;

    use quickcheck::{Arbitrary, Gen};
    use quickcheck_macros::quickcheck;

    impl Arbitrary for GoBoard {
        fn arbitrary<G: Gen>(g: &mut G) -> GoBoard {
            generation::generate_candidate(g)
        }
    }

    #[quickcheck]
    fn inverse(board: GoBoard) {
        assert_eq!(GoGame::from_sgf(&board.to_sgf()).board, board);
    }
}

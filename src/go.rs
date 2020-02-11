#![allow(dead_code)]

mod benson;
mod bit_board;
pub use bit_board::BitBoard;
use bit_board::{BOARD_HEIGHT, BOARD_WIDTH};

use im::conslist::ConsList;
use std::fmt;
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BoardCell {
    Empty,
    // OutOfBounds,
    Occupied(GoPlayer),
}

type BoardCoord = (usize, usize);

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GoPlayer {
    Black,
    White,
}

impl GoPlayer {
    pub fn flip(&self) -> GoPlayer {
        match self {
            GoPlayer::Black => GoPlayer::White,
            GoPlayer::White => GoPlayer::Black,
        }
    }
}

#[derive(PartialEq, Clone)]
pub struct GoBoard {
    white: BitBoard,
    black: BitBoard,
}

impl Debug for GoBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for j in 0..BOARD_HEIGHT {
            for i in 0..BOARD_WIDTH {
                f.write_str(match self.get_cell((i, j)) {
                    BoardCell::Empty => ".",
                    BoardCell::Occupied(GoPlayer::White) => "w",
                    BoardCell::Occupied(GoPlayer::Black) => "b",
                })?;
            }
            f.write_str("\n")?;
        }

        Ok(())
    }
}

impl GoBoard {
    fn empty() -> GoBoard {
        GoBoard {
            white: BitBoard::empty(),
            black: BitBoard::empty(),
        }
    }

    fn empty_cells(&self) -> BitBoard {
        !(self.white | self.black)
    }

    fn set_cell(&mut self, position: BoardCoord, cell: BoardCell) {
        let mask = BitBoard::singleton(position);

        match cell {
            BoardCell::Empty => {
                self.white = self.white & !mask;
                self.black = self.black & !mask;
            }
            BoardCell::Occupied(GoPlayer::Black) => {
                self.black = self.black | mask;
            }
            BoardCell::Occupied(GoPlayer::White) => {
                self.white = self.white | mask;
            }
        }
    }

    fn get_cell(&self, position: BoardCoord) -> BoardCell {
        let mask = BitBoard::singleton(position);

        if !((mask & self.white).is_empty()) {
            return BoardCell::Occupied(GoPlayer::White);
        }

        if !((mask & self.black).is_empty()) {
            return BoardCell::Occupied(GoPlayer::Black);
        }

        BoardCell::Empty
    }

    fn get_bitboard_for_player(&self, player: GoPlayer) -> BitBoard {
        match player {
            GoPlayer::Black => self.black,
            GoPlayer::White => self.white,
        }
    }

    fn get_bitboard_at_position(&self, position: BoardCoord) -> BitBoard {
        let mask = BitBoard::singleton(position);

        if !((mask & self.white).is_empty()) {
            return self.white;
        }

        if !((mask & self.black).is_empty()) {
            return self.black;
        }

        panic!("No board at this position")
    }

    fn group_has_liberties(&self, position: BoardCoord) -> bool {
        let mask = self.get_bitboard_at_position(position);

        let group = BitBoard::singleton(position).flood_fill(mask);

        !(group.expand_one() & self.empty_cells()).is_empty()
    }

    fn remove_group(&mut self, position: BoardCoord) {
        match self.get_cell(position) {
            BoardCell::Occupied(GoPlayer::White) => {
                self.white = self.white & !(BitBoard::singleton(position).flood_fill(self.white));
            }
            BoardCell::Occupied(GoPlayer::Black) => {
                self.black = self.black & !(BitBoard::singleton(position).flood_fill(self.black))
            }
            BoardCell::Empty => panic!("No group at location {:?}", position),
        }
    }
}

fn get_surrounding_positions(position: BoardCoord) -> Vec<BoardCoord> {
    let mut positions = Vec::with_capacity(4);

    if position.0 > 0 {
        positions.push((position.0 - 1, position.1));
    }

    if position.1 > 0 {
        positions.push((position.0, position.1 - 1));
    }

    if position.0 < (BOARD_WIDTH - 1) {
        positions.push((position.0 + 1, position.1));
    }

    if position.1 < (BOARD_HEIGHT - 1) {
        positions.push((position.0, position.1 + 1));
    }

    positions
}

#[derive(Debug, PartialEq)]
pub struct GoGame {
    boards: ConsList<GoBoard>,
    current_player: GoPlayer,
}

#[derive(Debug, PartialEq)]
pub enum MoveError {
    Occupied,
    OutOfBounds,
    OutOfTurn,
    Suicidal,
    Ko,
}

impl GoGame {
    pub fn empty() -> GoGame {
        let boards = ConsList::singleton(GoBoard::empty());

        GoGame {
            boards,
            current_player: GoPlayer::Black,
        }
    }

    pub fn get_board(&self) -> Arc<GoBoard> {
        self.boards.head().unwrap()
    }

    fn get_cell(&self, position: BoardCoord) -> BoardCell {
        self.get_board().get_cell(position)
    }

    pub fn play_move_for_player(
        &self,
        position: BoardCoord,
        player: GoPlayer,
    ) -> Result<GoGame, MoveError> {
        if self.current_player != player {
            return Err(MoveError::OutOfTurn);
        }

        self.play_move(position)
    }

    pub fn play_move(&self, position: BoardCoord) -> Result<GoGame, MoveError> {
        if self.get_cell(position) != BoardCell::Empty {
            return Err(MoveError::Occupied);
        }

        let mut new_board = self.get_board().as_ref().clone();
        new_board.set_cell(position, BoardCell::Occupied(self.current_player));

        // Remove adjacent groups with no liberties owned by other player
        for surrounding_position in get_surrounding_positions(position) {
            if new_board.get_cell(surrounding_position)
                == BoardCell::Occupied(self.current_player.flip())
            {
                if !new_board.group_has_liberties(surrounding_position) {
                    new_board.remove_group(surrounding_position);
                }
            }
        }

        // Evaluate suicide
        if !new_board.group_has_liberties(position) {
            return Err(MoveError::Suicidal);
        }

        // Evaluate ko
        if self.boards.iter().any(|board| *board == new_board) {
            return Err(MoveError::Ko);
        }

        Ok(GoGame {
            boards: self.boards.cons(new_board),
            current_player: self.current_player.flip(),
        })
    }

    pub fn generate_moves(&self) -> Vec<GoGame> {
        let mut games = Vec::new();

        for i in 0..BOARD_WIDTH {
            for j in 0..BOARD_HEIGHT {
                if let Ok(game) = self.play_move((i, j)) {
                    games.push(game);
                }
            }
        }

        games
    }
}

impl GoGame {
    pub fn from_sgf(sgf_string: &str) -> GoGame {
        use sgf_parser::{parse, Action, Color, SgfToken};
        use std::convert::TryInto;

        let sgf = parse(sgf_string).unwrap();

        // assert_eq!(sgf.count_variations(), 1);

        let mut game = GoGame::empty();

        for node in sgf.iter() {
            // TODO: Work out why we have to clone here
            for token in node.tokens.clone() {
                match token {
                    SgfToken::Move {
                        color,
                        action: Action::Move(i, j),
                    } => {
                        game = game
                            .play_move_for_player(
                                ((i - 1).try_into().unwrap(), (j - 1).try_into().unwrap()),
                                match color {
                                    Color::Black => GoPlayer::Black,
                                    Color::White => GoPlayer::White,
                                },
                            )
                            .unwrap()
                    }
                    _ => {}
                }
            }
        }

        game
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_add_stone() {
        let game = GoGame::empty();
        let game = game.play_move_for_player((0, 0), GoPlayer::Black).unwrap();

        assert_eq!(game.get_cell((0, 0)), BoardCell::Occupied(GoPlayer::Black));
    }

    #[test]
    fn previous_board_is_not_mutated() {
        let old_game = GoGame::empty();
        let new_game = old_game
            .play_move_for_player((0, 0), GoPlayer::Black)
            .unwrap();

        assert_ne!(old_game.get_cell((0, 0)), new_game.get_cell((0, 0)));
    }

    #[test]
    fn current_player_starts_as_black() {
        let game = GoGame::empty();

        assert_eq!(game.current_player, GoPlayer::Black);
    }

    #[test]
    fn player_advances_when_playing_move() {
        let game = GoGame::empty().play_move((0, 0)).unwrap();

        assert_eq!(game.current_player, GoPlayer::White);
    }

    #[test]
    fn cannot_play_move_out_of_turn() {
        let result = GoGame::empty().play_move_for_player((0, 0), GoPlayer::White);

        assert_eq!(result, Err(MoveError::OutOfTurn));
    }

    #[test]
    fn cannot_play_in_occupied_space() {
        let game = GoGame::empty().play_move((0, 0)).unwrap();
        let result = game.play_move((0, 0));

        assert_eq!(result, Err(MoveError::Occupied));
    }

    #[test]
    fn single_groups_are_captured() {
        let game = GoGame::from_sgf(include_str!("test_sgfs/single_groups_are_captured.sgf"));

        assert_eq!(game.get_cell((0, 0)), BoardCell::Empty);
    }

    #[test]
    fn complex_groups_are_captured() {
        let game = GoGame::from_sgf(include_str!("test_sgfs/complex_capture.sgf"));
        let game = game.play_move((11, 6)).unwrap();

        assert_eq!(game.get_cell((11, 5)), BoardCell::Empty);
        assert_eq!(game.get_cell((10, 5)), BoardCell::Empty);
        assert_eq!(game.get_cell((9, 5)), BoardCell::Empty);
        assert_eq!(game.get_cell((10, 4)), BoardCell::Empty);
        assert_eq!(game.get_cell((10, 3)), BoardCell::Empty);
        assert_eq!(game.get_cell((10, 2)), BoardCell::Empty);
        assert_eq!(game.get_cell((9, 3)), BoardCell::Empty);

        assert_eq!(game.get_cell((9, 4)), BoardCell::Occupied(GoPlayer::White));
        assert_eq!(game.get_cell((11, 4)), BoardCell::Occupied(GoPlayer::White));
    }

    #[test]
    fn capturing_has_precedence_over_suicide() {
        let game = GoGame::from_sgf(include_str!(
            "test_sgfs/capturing_has_precedence_over_suicide.sgf"
        ));

        assert_eq!(game.get_cell((1, 0)), BoardCell::Empty);
    }

    #[test]
    fn cannot_commit_suicide() {
        let game = GoGame::from_sgf(include_str!("test_sgfs/cannot_commit_suicide.sgf"));
        let result = game.play_move((0, 0));

        assert_eq!(result, Err(MoveError::Suicidal));
    }

    #[test]
    fn ko_rule_simple() {
        let game = GoGame::from_sgf(include_str!("test_sgfs/ko_rule_simple.sgf"));
        let result = game.play_move((2, 2));

        assert_eq!(result, Err(MoveError::Ko));
    }
}

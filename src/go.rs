#![allow(dead_code)]

use im::conslist::ConsList;
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BoardCell {
    Empty,
    OutOfBounds,
    Occupied(GoPlayer),
}

type BoardPosition = (usize, usize);

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

#[derive(Debug, PartialEq, Clone)]
pub struct GoBoard([[BoardCell; 19]; 19]);

impl GoBoard {
    fn empty() -> GoBoard {
        GoBoard([[BoardCell::Empty; 19]; 19])
    }

    fn set_cell(&mut self, position: BoardPosition, cell: BoardCell) {
        self.0[position.0][position.1] = cell;
    }

    fn get_cell(&self, position: BoardPosition) -> BoardCell {
        self.0[position.0][position.1]
    }

    fn group_has_liberties(
        &self,
        position: BoardPosition,
        visited: &mut HashSet<BoardPosition>,
    ) -> bool {
        visited.insert(position);

        if self.position_has_liberties(position) {
            return true;
        }

        if let BoardCell::Occupied(player) = self.get_cell(position) {
            for surrounding_position in get_surrounding_positions(position) {
                if self.get_cell(surrounding_position) == BoardCell::Occupied(player)
                    && !visited.contains(&surrounding_position)
                {
                    if self.group_has_liberties(surrounding_position, visited) {
                        return true;
                    }
                }
            }
            return false;
        } else {
            panic!("No group at location {:?}", position);
        }
    }

    fn position_has_liberties(&self, position: BoardPosition) -> bool {
        get_surrounding_positions(position)
            .iter()
            .any(|&surrounding_position| self.get_cell(surrounding_position) == BoardCell::Empty)
    }

    fn remove_group(&mut self, position: BoardPosition) {
        match self.get_cell(position) {
            BoardCell::Occupied(player) => {
                self.set_cell(position, BoardCell::Empty);
                for surrounding_position in get_surrounding_positions(position) {
                    if self.get_cell(surrounding_position) == BoardCell::Occupied(player) {
                        self.remove_group(surrounding_position);
                    }
                }
            }
            _ => panic!("No group at location {:?}", position),
        }
    }
}

fn get_surrounding_positions(position: BoardPosition) -> Vec<BoardPosition> {
    let mut positions = Vec::with_capacity(4);

    if position.0 > 0 {
        positions.push((position.0 - 1, position.1));
    }

    if position.1 > 0 {
        positions.push((position.0, position.1 - 1));
    }

    if position.0 < 18 {
        positions.push((position.0 + 1, position.1));
    }

    if position.1 < 18 {
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

    fn get_cell(&self, position: BoardPosition) -> BoardCell {
        self.get_board().get_cell(position)
    }

    pub fn play_move_for_player(
        &self,
        position: BoardPosition,
        player: GoPlayer,
    ) -> Result<GoGame, MoveError> {
        if self.current_player != player {
            return Err(MoveError::OutOfTurn);
        }

        self.play_move(position)
    }

    pub fn play_move(&self, position: BoardPosition) -> Result<GoGame, MoveError> {
        if self.get_cell(position) != BoardCell::Empty {
            return Err(MoveError::Occupied);
        }

        let mut new_board = self.get_board().as_ref().clone();
        new_board.set_cell(position, BoardCell::Occupied(self.current_player));

        // Remove adjacent groups with no liberties owned by other player
        let mut visited = HashSet::new();
        for surrounding_position in get_surrounding_positions(position) {
            if new_board.get_cell(surrounding_position)
                == BoardCell::Occupied(self.current_player.flip())
            {
                if !new_board.group_has_liberties(surrounding_position, &mut visited) {
                    new_board.remove_group(surrounding_position);
                }
            }
        }

        // Evaluate suicide
        if !new_board.group_has_liberties(position, &mut visited) {
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

        for i in 0..19 {
            for j in 0..19 {
                if let Ok(game) = self.play_move((i, j)) {
                    games.push(game);
                }
            }
        }

        games
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl GoGame {
        fn from_sgf(sgf_string: &str) -> GoGame {
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
    fn capturing_has_precedence_over_suicide() {
        let game = GoGame::from_sgf(include_str!("test_sgfs/capturing_has_precedence_over_suicide.sgf"));

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

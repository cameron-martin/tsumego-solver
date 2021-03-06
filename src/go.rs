//! Encapsulates the rules of the game of go.
//!
//! The current state of a game is represented by [`GoGame`](./struct.GoGame.html),
//! and moves can be played using [`GoGame::play_move`](./struct.GoGame.html#method.play_move)

mod benson;
mod bit_board;
mod sgf_conversion;
pub use bit_board::{BitBoard, BitBoardEdge, BoardPosition};
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::{Display, Write};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Move {
    Pass,
    Place(BoardPosition),
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Move::Pass => f.write_str("Pass"),
            Move::Place(position) => f.write_fmt(format_args!("{}", position)),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BoardCell {
    Empty,
    OutOfBounds,
    Occupied(GoPlayer),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum GoPlayer {
    Black,
    White,
}

impl GoPlayer {
    pub fn flip(self) -> GoPlayer {
        match self {
            GoPlayer::Black => GoPlayer::White,
            GoPlayer::White => GoPlayer::Black,
        }
    }

    pub fn both() -> impl Iterator<Item = &'static GoPlayer> {
        [GoPlayer::Black, GoPlayer::White].iter()
    }
}

impl Display for GoPlayer {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            GoPlayer::Black => f.write_str("black"),
            GoPlayer::White => f.write_str("white"),
        }
    }
}

// Being set in both black and white denotes "out of bounds"
#[derive(PartialEq, Clone, Copy, Debug, Eq, Hash)]
pub struct GoBoard {
    white: BitBoard,
    black: BitBoard,
}

impl Display for GoBoard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for j in 0..BitBoard::height() {
            for i in 0..BitBoard::width() {
                if i != 0 {
                    f.write_char(' ')?;
                }

                f.write_char(match self.get_cell(BoardPosition::new(i, j)) {
                    BoardCell::Empty => '.',
                    BoardCell::OutOfBounds => '_',
                    BoardCell::Occupied(GoPlayer::White) => 'w',
                    BoardCell::Occupied(GoPlayer::Black) => 'b',
                })?;
            }
            f.write_str("\n")?;
        }

        Ok(())
    }
}

impl GoBoard {
    pub fn empty() -> GoBoard {
        GoBoard {
            white: BitBoard::empty(),
            black: BitBoard::empty(),
        }
    }

    pub fn new(black: BitBoard, white: BitBoard, out_of_bounds: BitBoard) -> GoBoard {
        GoBoard {
            white: white | out_of_bounds,
            black: black | out_of_bounds,
        }
    }

    /// Empty cells, including out of bounds
    pub fn empty_cells(&self) -> BitBoard {
        !(self.white ^ self.black)
    }

    pub fn out_of_bounds(&self) -> BitBoard {
        self.white & self.black
    }

    pub fn set_cell(&mut self, position: BoardPosition, cell: BoardCell) {
        let mask = BitBoard::singleton(position);

        match cell {
            BoardCell::Empty => {
                self.black = self.black & !mask;
                self.white = self.white & !mask;
            }
            BoardCell::Occupied(GoPlayer::Black) => {
                self.black = self.black | mask;
                self.white = self.white & !mask;
            }
            BoardCell::Occupied(GoPlayer::White) => {
                self.black = self.black & !mask;
                self.white = self.white | mask;
            }
            BoardCell::OutOfBounds => {
                self.black = self.black | mask;
                self.white = self.white | mask;
            }
        }
    }

    pub fn get_cell(&self, position: BoardPosition) -> BoardCell {
        let mask = BitBoard::singleton(position);

        if !((mask & self.out_of_bounds()).is_empty()) {
            return BoardCell::OutOfBounds;
        }

        if !((mask & self.white).is_empty()) {
            return BoardCell::Occupied(GoPlayer::White);
        }

        if !((mask & self.black).is_empty()) {
            return BoardCell::Occupied(GoPlayer::Black);
        }

        BoardCell::Empty
    }

    pub fn get_bitboard_for_player(&self, player: GoPlayer) -> BitBoard {
        match player {
            GoPlayer::Black => (self.black & !self.white),
            GoPlayer::White => (self.white & !self.black),
        }
    }

    fn set_bitboard_for_player(&mut self, player: GoPlayer, board: BitBoard) {
        match player {
            GoPlayer::Black => self.black = board | self.out_of_bounds(),
            GoPlayer::White => self.white = board | self.out_of_bounds(),
        }
    }

    fn get_bitboard_at_position(&self, position: BoardPosition) -> BitBoard {
        let mask = BitBoard::singleton(position);
        let white = self.get_bitboard_for_player(GoPlayer::White);
        let black = self.get_bitboard_for_player(GoPlayer::Black);

        if !((mask & white).is_empty()) {
            return white;
        }

        if !((mask & black).is_empty()) {
            return black;
        }

        panic!("No board at this position")
    }

    fn group_has_liberties(&self, position: BoardPosition) -> bool {
        let mask = self.get_bitboard_at_position(position);

        let group = BitBoard::singleton(position).flood_fill(mask);

        !(group.expand_one() & self.empty_cells()).is_empty()
    }

    fn get_alive_groups_for_player(&self, player: GoPlayer) -> BitBoard {
        let bitboard = self.get_bitboard_for_player(player);

        (self.empty_cells().expand_one() & bitboard).flood_fill(bitboard)
    }

    fn remove_dead_groups_for_player(&mut self, player: GoPlayer) {
        let stones_with_liberties = self.get_alive_groups_for_player(player);

        self.set_bitboard_for_player(player, stones_with_liberties);
    }

    pub fn has_dead_groups(&self) -> bool {
        GoPlayer::both().any(|&player| {
            let alive_groups = self.get_alive_groups_for_player(player);

            self.get_bitboard_for_player(player) != alive_groups
        })
    }

    fn is_out_of_bounds(&self, position: BoardPosition) -> bool {
        !(BitBoard::singleton(position) & self.out_of_bounds()).is_empty()
    }

    pub fn set_out_of_bounds(&mut self, out_of_bounds: BitBoard) {
        let prev_out_of_bounds = self.out_of_bounds();
        self.white = (self.white & !prev_out_of_bounds) | out_of_bounds;
        self.black = (self.black & !prev_out_of_bounds) | out_of_bounds;
    }

    pub fn stable_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);

        hasher.finish()
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
pub enum PassState {
    NoPass,
    PassedOnce,
    PassedTwice,
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
pub struct GoGame {
    ko_violations: BitBoard,

    pub board: GoBoard,

    /// The player whose turn it currently is.
    pub current_player: GoPlayer,

    /// The number of sequential passes that have occured.
    ///
    /// After two sequential passes have occured, the game has ended.
    pub pass_state: PassState,
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
    pub fn empty(current_player: GoPlayer) -> GoGame {
        GoGame {
            board: GoBoard::empty(),
            ko_violations: BitBoard::empty(),
            current_player,
            pass_state: PassState::NoPass,
        }
    }

    pub fn from_board(board: GoBoard, current_player: GoPlayer) -> GoGame {
        GoGame {
            board,
            ko_violations: BitBoard::empty(),
            current_player,
            pass_state: PassState::NoPass,
        }
    }

    fn get_cell(&self, position: BoardPosition) -> BoardCell {
        self.board.get_cell(position)
    }

    fn is_out_of_bounds(&self, position: BoardPosition) -> bool {
        self.board.is_out_of_bounds(position)
    }

    pub fn play_move_for_player(
        &self,
        go_move: Move,
        player: GoPlayer,
    ) -> Result<GoGame, MoveError> {
        if self.current_player != player {
            return Err(MoveError::OutOfTurn);
        }

        self.play_move(go_move)
    }

    /// Plays any type of move: either placing a stone or passing.
    ///
    /// This is a combination of [`GoGame::place_stone`](#method.place_stone) and [`GoGame::pass`](#method.pass),
    /// useful when wanting to treat both move types in a uniform way.
    ///
    /// ```rust
    /// use tsumego_solver::go::{GoGame, GoPlayer, Move};
    ///
    /// let game = GoGame::empty(GoPlayer::Black).play_move(Move::Pass).unwrap();
    ///
    /// assert_eq!(game.current_player, GoPlayer::White);
    /// ```
    pub fn play_move(&self, go_move: Move) -> Result<GoGame, MoveError> {
        match go_move {
            Move::Place(position) => self.place_stone(position),
            Move::Pass => Ok(self.pass()),
        }
    }

    /// Plays a move that involves placing a stone on the board.
    ///
    /// Returns an error if the move is invalid, usually by the rules of the game being violated.
    ///
    /// ```rust
    /// use tsumego_solver::go::{GoGame, BoardPosition, BoardCell, GoPlayer};
    ///
    /// let game = GoGame::empty(GoPlayer::Black).place_stone(BoardPosition::new(0, 0)).unwrap();
    ///
    /// assert_eq!(game.board.get_cell(BoardPosition::new(0, 0)), BoardCell::Occupied(GoPlayer::Black));
    /// ```
    pub fn place_stone(&self, position: BoardPosition) -> Result<GoGame, MoveError> {
        if self.get_cell(position) != BoardCell::Empty {
            return Err(MoveError::Occupied);
        }

        if self.is_out_of_bounds(position) {
            return Err(MoveError::OutOfBounds);
        }

        let next_player = self.current_player.flip();

        let mut new_board = self.board;
        new_board.set_cell(position, BoardCell::Occupied(self.current_player));

        // Remove dead groups owned by other player
        new_board.remove_dead_groups_for_player(next_player);

        // Evaluate suicide
        if !new_board.group_has_liberties(position) {
            return Err(MoveError::Suicidal);
        }

        // Evaluate ko
        if self.ko_violations.is_set(position) {
            return Err(MoveError::Ko);
        }

        let ko_violations = if (BitBoard::singleton(position).immediate_exterior()
            & self.board.get_bitboard_for_player(self.current_player))
        .is_empty()
        {
            (self.board.get_bitboard_for_player(next_player)
                & !new_board.get_bitboard_for_player(next_player))
            .singletons()
        } else {
            BitBoard::empty()
        };

        Ok(GoGame {
            ko_violations,
            board: new_board,
            current_player: next_player,
            pass_state: PassState::NoPass,
        })
    }

    /// Plays a pass.
    ///
    /// This advances the current player, but does not add any stones to the board.
    ///
    /// ```rust
    /// use tsumego_solver::go::{GoGame, GoPlayer};
    ///
    /// let game = GoGame::empty(GoPlayer::Black).pass();
    ///
    /// assert_eq!(game.current_player, GoPlayer::White);
    /// ```
    pub fn pass(&self) -> GoGame {
        GoGame {
            board: self.board,
            ko_violations: BitBoard::empty(),
            current_player: self.current_player.flip(),
            pass_state: match self.pass_state {
                PassState::NoPass => PassState::PassedOnce,
                PassState::PassedOnce => PassState::PassedTwice,
                PassState::PassedTwice => panic!("Cannot pass when the game is finished"),
            },
        }
    }

    fn generate_placing_moves(&self) -> Vec<(GoGame, Move)> {
        let mut games = Vec::new();

        for position in (!(self.board.white | self.board.black)).positions() {
            if let Ok(game) = self.place_stone(position) {
                games.push((game, Move::Place(position)));
            }
        }

        games
    }

    /// Generates all legal moves and their resulting board states.
    pub fn generate_moves(&self) -> Vec<(GoGame, Move)> {
        let mut games = self.generate_placing_moves();

        games.push((self.pass(), Move::Pass));

        games
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_add_stone() {
        let game = GoGame::empty(GoPlayer::Black);
        let game = game
            .play_move_for_player(Move::Place(BoardPosition::new(0, 0)), GoPlayer::Black)
            .unwrap();

        assert_eq!(
            game.get_cell(BoardPosition::new(0, 0)),
            BoardCell::Occupied(GoPlayer::Black)
        );
    }

    #[test]
    fn previous_board_is_not_mutated() {
        let old_game = GoGame::empty(GoPlayer::Black);
        let new_game = old_game
            .play_move_for_player(Move::Place(BoardPosition::new(0, 0)), GoPlayer::Black)
            .unwrap();

        assert_ne!(
            old_game.get_cell(BoardPosition::new(0, 0)),
            new_game.get_cell(BoardPosition::new(0, 0))
        );
    }

    #[test]
    fn current_player_starts_as_specified() {
        let game = GoGame::empty(GoPlayer::Black);

        assert_eq!(game.current_player, GoPlayer::Black);
    }

    #[test]
    fn player_advances_when_playing_move() {
        let game = GoGame::empty(GoPlayer::Black)
            .place_stone(BoardPosition::new(0, 0))
            .unwrap();

        assert_eq!(game.current_player, GoPlayer::White);
    }

    #[test]
    fn cannot_play_move_out_of_turn() {
        let result = GoGame::empty(GoPlayer::Black)
            .play_move_for_player(Move::Place(BoardPosition::new(0, 0)), GoPlayer::White);

        assert_eq!(result, Err(MoveError::OutOfTurn));
    }

    #[test]
    fn cannot_play_in_occupied_space() {
        let game = GoGame::empty(GoPlayer::Black)
            .place_stone(BoardPosition::new(0, 0))
            .unwrap();
        let result = game.place_stone(BoardPosition::new(0, 0));

        assert_eq!(result, Err(MoveError::Occupied));
    }

    #[test]
    fn single_groups_are_captured() {
        let game = GoGame::from_sgf(include_str!("test_sgfs/single_groups_are_captured.sgf"));

        assert_eq!(game.get_cell(BoardPosition::new(0, 0)), BoardCell::Empty);
    }

    #[test]
    fn complex_groups_are_captured() {
        let game = GoGame::from_sgf(include_str!("test_sgfs/complex_capture.sgf"));
        let game = game.place_stone(BoardPosition::new(11, 6)).unwrap();

        assert_eq!(
            format!("{}", game.board),
            ". . . . . . . . . . . . . . . .\n\
             . b . b . b b w w . w . . . . .\n\
             . . . . . . . b w w . w . w . .\n\
             . . . b . b b . w . . w b . . .\n\
             . . . . b . . . w w . w . . . .\n\
             . . . b . . w . w . . . w . . .\n\
             . . . . . . . . . w w w . . . .\n\
             . . . . . . . . . . . . . . . .\n"
        );
    }

    #[test]
    fn capturing_has_precedence_over_suicide() {
        let game = GoGame::from_sgf(include_str!(
            "test_sgfs/capturing_has_precedence_over_suicide.sgf"
        ));

        assert_eq!(game.get_cell(BoardPosition::new(1, 0)), BoardCell::Empty);
    }

    #[test]
    fn cannot_commit_suicide() {
        let game = GoGame::from_sgf(include_str!("test_sgfs/cannot_commit_suicide.sgf"));
        let result = game.place_stone(BoardPosition::new(0, 0));

        assert_eq!(result, Err(MoveError::Suicidal));
    }

    #[test]
    fn ko_rule_simple() {
        let game = GoGame::from_sgf(include_str!("test_sgfs/ko_rule_simple.sgf"));
        let result = game.place_stone(BoardPosition::new(2, 2));

        assert_eq!(result, Err(MoveError::Ko));
    }

    #[test]
    fn capture_two_recapture_one_not_ko_violation() {
        let game = GoGame::from_sgf(include_str!("test_sgfs/capture_two_recapture_one.sgf"));

        game.place_stone(BoardPosition::new(3, 2)).unwrap();
    }

    #[test]
    fn capturing_single_and_joining_group_does_not_trigger_ko() {
        let game = GoGame::from_sgf(include_str!("test_sgfs/capture_single_join_group.sgf"));

        let result = game.place_stone(BoardPosition::new(2, 1));

        assert!(result.is_ok());
    }

    #[test]
    fn out_of_bounds_moves_are_not_generated() {
        let game = GoGame::from_sgf(include_str!("test_sgfs/puzzles/true_simple1.sgf"));
        let moves = game.generate_moves();

        assert_eq!(moves.len(), 6);
    }

    #[test]
    fn pass_sets_last_move_pass() {
        let game = GoGame::from_sgf(include_str!("test_sgfs/ko_rule_simple.sgf"));
        let game = game.pass();

        assert_eq!(game.pass_state, PassState::PassedOnce);
    }

    #[test]
    fn move_clears_last_move_pass() {
        let game = GoGame::from_sgf(include_str!("test_sgfs/ko_rule_simple.sgf"));
        let game = game.pass();
        let game = game.place_stone(BoardPosition::new(13, 7)).unwrap();

        assert_eq!(game.pass_state, PassState::NoPass);
    }

    #[test]
    fn pass_advances_player() {
        let game = GoGame::from_sgf(include_str!("test_sgfs/ko_rule_simple.sgf"));
        let new_game = game.pass();

        assert_ne!(game.current_player, new_game.current_player);
    }

    #[test]
    fn has_dead_groups_black() {
        let mut game = GoBoard::empty();
        game.set_cell(
            BoardPosition::new(0, 0),
            BoardCell::Occupied(GoPlayer::Black),
        );
        game.set_cell(
            BoardPosition::new(0, 1),
            BoardCell::Occupied(GoPlayer::White),
        );
        game.set_cell(
            BoardPosition::new(1, 0),
            BoardCell::Occupied(GoPlayer::White),
        );

        assert!(game.has_dead_groups());
    }

    #[test]
    fn has_dead_groups_white() {
        let mut game = GoBoard::empty();
        game.set_cell(
            BoardPosition::new(0, 0),
            BoardCell::Occupied(GoPlayer::White),
        );
        game.set_cell(
            BoardPosition::new(0, 1),
            BoardCell::Occupied(GoPlayer::Black),
        );
        game.set_cell(
            BoardPosition::new(1, 0),
            BoardCell::Occupied(GoPlayer::Black),
        );

        assert!(game.has_dead_groups());
    }

    #[test]
    fn has_dead_groups_false() {
        let mut game = GoBoard::empty();
        game.set_cell(
            BoardPosition::new(0, 1),
            BoardCell::Occupied(GoPlayer::White),
        );
        game.set_cell(
            BoardPosition::new(1, 0),
            BoardCell::Occupied(GoPlayer::White),
        );

        assert!(!game.has_dead_groups());
    }

    #[test]
    fn hashing_is_stable() {
        assert_eq!(GoBoard::empty().stable_hash(), 13284472273662876477);
    }
}

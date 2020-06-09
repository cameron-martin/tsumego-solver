use crate::{
    go::{BoardCell, BoardPosition, GoBoard, GoGame, GoPlayer},
    puzzle::{NoProfile, Puzzle},
};
use pest::{iterators::Pair, Parser};
use std::error::Error;

#[derive(Parser)]
#[grammar = "gotools_parser/grammar.pest"]
struct GoToolsParser;

struct StoneSet {
    points: Vec<(u8, u8, GoPlayer)>,
}

impl StoneSet {
    fn is_in_bounds(&self, x_len: u8, y_len: u8) -> bool {
        self.points
            .iter()
            .all(|point| point.0 < x_len && point.1 < y_len)
    }

    fn rotate(&mut self) {
        for point in self.points.iter_mut() {
            *point = (point.1, 18 - point.0, point.2);
        }
    }
}

pub struct PuzzleCollection {
    pub total_puzzles: u32,
    pub valid_puzzles: Vec<Puzzle<NoProfile>>,
}

impl PuzzleCollection {
    pub fn new() -> PuzzleCollection {
        PuzzleCollection {
            total_puzzles: 0,
            valid_puzzles: Vec::new(),
        }
    }

    pub fn append(&mut self, mut other: PuzzleCollection) {
        self.total_puzzles += other.total_puzzles;
        self.valid_puzzles.append(&mut other.valid_puzzles);
    }
}

fn char_to_int(character: char) -> u8 {
    (character as u8) - ('A' as u8)
}

fn read_puzzle(pair: Pair<Rule>) -> Option<Puzzle<NoProfile>> {
    let puzzle_definition = pair
        .into_inner()
        .find(|inner_pair| inner_pair.as_rule() == Rule::puzzle_definition)
        .unwrap();

    let stones = puzzle_definition
        .into_inner()
        .filter(|pair| pair.as_rule() == Rule::stone);

    let mut board = GoBoard::empty();

    let mut stone_set = StoneSet {
        points: stones
            .map(|stone| {
                let mut chars = stone.as_str().chars();

                let player = if chars.next().unwrap() == '?' {
                    GoPlayer::White
                } else {
                    GoPlayer::Black
                };

                (
                    char_to_int(chars.next().unwrap()),
                    char_to_int(chars.next().unwrap()),
                    player,
                )
            })
            .collect(),
    };

    let mut rotations = 0;

    while !stone_set.is_in_bounds(16, 8) {
        stone_set.rotate();

        if rotations > 3 {
            return None;
        }

        rotations += 1;
    }

    for (x, y, player) in stone_set.points {
        board.set_cell(BoardPosition::new(x, y), BoardCell::Occupied(player));
    }

    let out_of_bounds = board
        .empty_cells()
        .groups()
        .max_by_key(|group| group.count())
        .unwrap();
    board.set_out_of_bounds(out_of_bounds);

    Some(Puzzle::new(GoGame::from_board(board, GoPlayer::Black)))
}

pub fn parse(contents: &str) -> Result<PuzzleCollection, Box<dyn Error>> {
    let top_pair = GoToolsParser::parse(Rule::file, contents)?.next().unwrap();

    let puzzle_pairs = top_pair
        .into_inner()
        .filter(|pair| pair.as_rule() == Rule::puzzle);

    let mut total_puzzles = 0;
    let mut valid_puzzles = Vec::new();
    for pair in puzzle_pairs {
        if let Some(puzzle) = read_puzzle(pair) {
            valid_puzzles.push(puzzle);
        }
        total_puzzles += 1;
    }

    Ok(PuzzleCollection {
        valid_puzzles,
        total_puzzles,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;
    use std::borrow::Borrow;

    #[test]
    fn parses_example_correctly() {
        let contents = include_str!("example_puzzles");

        let puzzles = parse(contents).unwrap();

        let mut string = String::new();

        for puzzle in puzzles.valid_puzzles {
            string.push_str(format!("{}\n", puzzle.current_game().board).borrow());
        }

        assert_snapshot!(string);
    }
}

use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{BitAnd, BitOr, BitXor, Not};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BoardPosition(u8);

impl BoardPosition {
    pub fn new(column: u8, row: u8) -> BoardPosition {
        BoardPosition(column + BitBoard::width() * row)
    }

    pub fn to_pair(self) -> (u8, u8) {
        let y = self.0 / BitBoard::width();

        let x = self.0 - (BitBoard::width() * y);

        (x, y)
    }
}

impl Display for BoardPosition {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        let (x, y) = self.to_pair();

        f.write_fmt(format_args!("({}, {})", x, y))
    }
}

/// A bitboard with 16 columns and 8 rows,
/// flowing left to right, then wrapping top to bottom.
#[derive(Copy, Clone, PartialEq)]
pub struct BitBoard(u128);

impl Debug for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..Self::height() {
            let row = (self.0 << (i * Self::width())) >> ((Self::height() - 1) * Self::width());
            f.write_str(&(format!("{:016b}", row) + "\n"))?;
        }

        Ok(())
    }
}

impl BitAnd for BitBoard {
    type Output = BitBoard;

    fn bitand(self, rhs: BitBoard) -> BitBoard {
        BitBoard(self.0 & rhs.0)
    }
}

impl BitOr for BitBoard {
    type Output = BitBoard;

    fn bitor(self, rhs: BitBoard) -> BitBoard {
        BitBoard(self.0 | rhs.0)
    }
}

impl Not for BitBoard {
    type Output = BitBoard;

    fn not(self) -> BitBoard {
        BitBoard(!self.0)
    }
}

impl BitXor for BitBoard {
    type Output = BitBoard;

    fn bitxor(self, rhs: BitBoard) -> BitBoard {
        BitBoard(self.0 ^ rhs.0)
    }
}

impl BitBoard {
    pub fn width() -> u8 {
        16
    }

    pub fn height() -> u8 {
        8
    }

    pub fn singleton(position: BoardPosition) -> BitBoard {
        BitBoard(0x8000_0000_0000_0000_0000_0000_0000_0000 >> position.0)
    }

    pub fn from_uint(int: u128) -> BitBoard {
        BitBoard(int)
    }

    pub fn top_edge() -> BitBoard {
        BitBoard(0xFFFF_0000_0000_0000_0000_0000_0000_0000u128)
    }

    pub fn bottom_edge() -> BitBoard {
        BitBoard(0x0000_0000_0000_0000_0000_0000_0000_FFFFu128)
    }

    pub fn right_edge() -> BitBoard {
        BitBoard(0x0001_0001_0001_0001_0001_0001_0001_0001u128)
    }

    pub fn left_edge() -> BitBoard {
        BitBoard(0x8000_8000_8000_8000_8000_8000_8000_8000u128)
    }

    pub fn empty() -> BitBoard {
        BitBoard(0)
    }

    pub fn shift_up(self) -> BitBoard {
        BitBoard(self.0 << Self::width())
    }

    pub fn shift_down(self) -> BitBoard {
        BitBoard(self.0 >> Self::width())
    }

    pub fn shift_left(self) -> BitBoard {
        BitBoard(self.0 << 1) & !Self::right_edge()
    }

    pub fn shift_right(self) -> BitBoard {
        BitBoard(self.0 >> 1) & !Self::left_edge()
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub fn is_set(self, position: BoardPosition) -> bool {
        !(self & Self::singleton(position)).is_empty()
    }

    pub fn set(self, position: BoardPosition) -> BitBoard {
        self | Self::singleton(position)
    }

    pub fn flood_fill(self, mask: BitBoard) -> BitBoard {
        let mut filled = self & mask;

        loop {
            let snapshot = filled;

            filled = filled.expand_one() & mask;

            if filled == snapshot {
                return filled;
            }
        }
    }

    /// Expands the set bits in all directions (left, right, up & down) by one cell
    pub fn expand_one(self) -> BitBoard {
        self | self.shift_up() | self.shift_down() | self.shift_left() | self.shift_right()
    }

    pub fn interior(self) -> BitBoard {
        self & (self.shift_up() | Self::bottom_edge())
            & (self.shift_down() | Self::top_edge())
            & (self.shift_left() | Self::right_edge())
            & (self.shift_right() | Self::left_edge())
    }

    pub fn border(self) -> BitBoard {
        self & !self.interior()
    }

    pub fn immediate_exterior(self) -> BitBoard {
        self.expand_one() & !self
    }

    pub fn groups(self) -> BitBoardGroupIterator {
        BitBoardGroupIterator {
            remaining_groups: self,
        }
    }

    pub fn positions(self) -> BitBoardPositionIterator {
        BitBoardPositionIterator {
            remaining_positions: self,
        }
    }

    pub fn some_cell(self) -> BoardPosition {
        BoardPosition(self.0.leading_zeros() as u8)
    }

    // Gets all single points on the board
    pub fn singletons(self) -> BitBoard {
        self & !self.shift_up() & !self.shift_down() & !self.shift_left() & !self.shift_right()
    }

    pub fn count(self) -> u32 {
        self.0.count_ones()
    }
}

pub struct BitBoardGroupIterator {
    remaining_groups: BitBoard,
}

impl Iterator for BitBoardGroupIterator {
    type Item = BitBoard;

    fn next(&mut self) -> Option<BitBoard> {
        if self.remaining_groups.is_empty() {
            None
        } else {
            let some_group = BitBoard::singleton(self.remaining_groups.some_cell())
                .flood_fill(self.remaining_groups);

            self.remaining_groups = self.remaining_groups & !some_group;

            Some(some_group)
        }
    }
}

pub struct BitBoardPositionIterator {
    remaining_positions: BitBoard,
}

impl Iterator for BitBoardPositionIterator {
    type Item = BoardPosition;

    fn next(&mut self) -> Option<BoardPosition> {
        if self.remaining_positions.is_empty() {
            None
        } else {
            let position = self.remaining_positions.some_cell();

            self.remaining_positions = self.remaining_positions & !BitBoard::singleton(position);

            Some(position)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use quickcheck::{Arbitrary, Gen};
    use quickcheck_macros::quickcheck;

    impl Arbitrary for BoardPosition {
        fn arbitrary<G: Gen>(g: &mut G) -> BoardPosition {
            BoardPosition((g.next_u32() % 128) as u8)
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            Box::new(self.0.shrink().map(BoardPosition))
        }
    }

    #[test]
    fn flood_fill() {
        let mask = BitBoard::from_uint(0b0000000000000000_0101011000000000_0000000100100000_0001011001101000_0000100000100000_0001000001110000_0000000000000000_0000000000000000);

        assert_eq!(
            format!("{:?}", mask),
            "0000000000000000\n\
             0101011000000000\n\
             0000000100100000\n\
             0001011001101000\n\
             0000100000100000\n\
             0001000001110000\n\
             0000000000000000\n\
             0000000000000000\n"
        );

        let filled = BitBoard::singleton(BoardPosition::new(11, 5)).flood_fill(mask);

        assert_eq!(
            format!("{:?}", filled),
            "0000000000000000\n\
             0000000000000000\n\
             0000000000100000\n\
             0000000001100000\n\
             0000000000100000\n\
             0000000001110000\n\
             0000000000000000\n\
             0000000000000000\n"
        );
    }

    #[test]
    fn from_uint() {
        let board = BitBoard::from_uint(0b0000000000000000_0000000000000000_0010000000000000_0000000000000000_0000000000000000_0000000100000000_0000000000000000_0000000000000000u128);

        assert_eq!(
            format!("{:?}", board),
            "0000000000000000\n\
             0000000000000000\n\
             0010000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000100000000\n\
             0000000000000000\n\
             0000000000000000\n"
        );
    }

    #[test]
    fn shifts() {
        let board = BitBoard::from_uint(0b1100011111000001_1000001110000001_0000000000000011_0000000000000111_1110000000000011_1110000000000000_1110000111110000_1110000111110000);

        assert_eq!(
            format!("{:?}", board),
            "1100011111000001\n\
             1000001110000001\n\
             0000000000000011\n\
             0000000000000111\n\
             1110000000000011\n\
             1110000000000000\n\
             1110000111110000\n\
             1110000111110000\n"
        );

        assert_eq!(
            format!("{:?}", board.shift_right()),
            "0110001111100000\n\
             0100000111000000\n\
             0000000000000001\n\
             0000000000000011\n\
             0111000000000001\n\
             0111000000000000\n\
             0111000011111000\n\
             0111000011111000\n"
        );

        assert_eq!(
            format!("{:?}", board.shift_left()),
            "1000111110000010\n\
             0000011100000010\n\
             0000000000000110\n\
             0000000000001110\n\
             1100000000000110\n\
             1100000000000000\n\
             1100001111100000\n\
             1100001111100000\n"
        );

        assert_eq!(
            format!("{:?}", board.shift_up()),
            "1000001110000001\n\
             0000000000000011\n\
             0000000000000111\n\
             1110000000000011\n\
             1110000000000000\n\
             1110000111110000\n\
             1110000111110000\n\
             0000000000000000\n"
        );

        assert_eq!(
            format!("{:?}", board.shift_down()),
            "0000000000000000\n\
             1100011111000001\n\
             1000001110000001\n\
             0000000000000011\n\
             0000000000000111\n\
             1110000000000011\n\
             1110000000000000\n\
             1110000111110000\n"
        );
    }

    #[test]
    fn shift_right_on_edge() {
        let board = BitBoard::from_uint(0b0000000000000000_0000000000000000_0000000000000001_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000);

        assert!(board.shift_right().is_empty());
    }

    #[quickcheck]
    fn singleton_some_cell_inverse(position: BoardPosition) {
        let board = BitBoard::singleton(position);

        assert_eq!(board.some_cell(), position);
    }

    #[test]
    fn iterate_groups() {
        let board = BitBoard::from_uint(0b0000000000000000_0100011000000000_0000000000000000_0000111001100000_0000100000100000_0000000001110000_0000000000000000_0000000000000000);

        assert_eq!(
            format!("{:?}", board),
            "0000000000000000\n\
             0100011000000000\n\
             0000000000000000\n\
             0000111001100000\n\
             0000100000100000\n\
             0000000001110000\n\
             0000000000000000\n\
             0000000000000000\n"
        );

        let mut iterator = board.groups();

        assert_eq!(
            format!("{:?}", iterator.next().unwrap()),
            "0000000000000000\n\
             0100000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n"
        );

        assert_eq!(
            format!("{:?}", iterator.next().unwrap()),
            "0000000000000000\n\
             0000011000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n"
        );

        assert_eq!(
            format!("{:?}", iterator.next().unwrap()),
            "0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000111000000000\n\
             0000100000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n"
        );

        assert_eq!(
            format!("{:?}", iterator.next().unwrap()),
            "0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000001100000\n\
             0000000000100000\n\
             0000000001110000\n\
             0000000000000000\n\
             0000000000000000\n"
        );

        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn iterate_groups2() {
        let board = BitBoard::from_uint(2596148429267413814265248164610048u128);

        assert_eq!(
            format!("{:?}", board),
            "0000000000000000\n\
             1000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n"
        );

        let mut iterator = board.groups();

        assert_eq!(
            format!("{:?}", iterator.next().unwrap()),
            "0000000000000000\n\
             1000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n"
        );

        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn edges_are_in_interior() {
        let board = BitBoard::from_uint(0b1100011111000001_1000001110000001_0000000000000011_0000000000000111_1110000000000011_1110000000000000_1110000111110000_1110000111110000);

        assert_eq!(
            format!("{:?}", board),
            "1100011111000001\n\
             1000001110000001\n\
             0000000000000011\n\
             0000000000000111\n\
             1110000000000011\n\
             1110000000000000\n\
             1110000111110000\n\
             1110000111110000\n"
        );

        assert_eq!(
            format!("{:?}", board.interior()),
            "1000001110000000\n\
             0000000000000000\n\
             0000000000000001\n\
             0000000000000011\n\
             0000000000000000\n\
             1100000000000000\n\
             1100000000000000\n\
             1100000011100000\n"
        );
    }

    #[test]
    fn iterate_positions() {
        let board = BitBoard::from_uint(0b0000000000000000_0110000000000000_0000000000000000_0000000001000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000);

        assert_eq!(
            format!("{:?}", board),
            "0000000000000000\n\
             0110000000000000\n\
             0000000000000000\n\
             0000000001000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n"
        );

        let mut iterator = board.positions();

        assert_eq!(
            format!("{:?}", BitBoard::singleton(iterator.next().unwrap())),
            "0000000000000000\n\
             0100000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n"
        );

        assert_eq!(
            format!("{:?}", BitBoard::singleton(iterator.next().unwrap())),
            "0000000000000000\n\
             0010000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n"
        );

        assert_eq!(
            format!("{:?}", BitBoard::singleton(iterator.next().unwrap())),
            "0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000001000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n"
        );

        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn singletons() {
        let board = BitBoard::from_uint(0b1000000000000000_0110000000000000_0000000000010000_0000000000000000_0000000000000000_0000000000000000_1000000000000000_1100000000000000);

        assert_eq!(
            format!("{:?}", board),
            "1000000000000000\n\
             0110000000000000\n\
             0000000000010000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             1000000000000000\n\
             1100000000000000\n"
        );

        assert_eq!(
            format!("{:?}", board.singletons()),
            "1000000000000000\n\
             0000000000000000\n\
             0000000000010000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n"
        );
    }

    #[test]
    fn some_cell() {
        let board = BitBoard::from_uint(0b0000000000000000_0110000000000000_0000000000010000_0000000000000000_0000000000000000_0000000000000000_1000000000000000_1100000000000000);

        assert_eq!(board.some_cell(), BoardPosition::new(1, 1));

        let board = BitBoard::from_uint(0b0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000010000000_0000000000000000_1000000000000000_0000000000000000);

        assert_eq!(board.some_cell(), BoardPosition::new(8, 4));

        let board = BitBoard::from_uint(0b0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000001000000000_0000010000000000);

        assert_eq!(board.some_cell(), BoardPosition::new(6, 6));
    }

    #[test]
    fn singleton() {
        assert_eq!(
            format!("{:?}", BitBoard::singleton(BoardPosition::new(8, 0))),
            "0000000010000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n"
        );

        assert_eq!(
            format!("{:?}", BitBoard::singleton(BoardPosition::new(4, 6))),
            "0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000100000000000\n\
             0000000000000000\n"
        );
    }

    #[test]
    fn set() {
        assert_eq!(
            format!("{:?}", BitBoard::empty().set(BoardPosition::new(1, 1))),
            "0000000000000000\n\
             0100000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n"
        );
    }
}

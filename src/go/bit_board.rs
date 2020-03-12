use std::arch::x86_64::*;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::mem;
use std::ops::{BitAnd, BitOr, Not};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BoardPosition(BitBoard);

impl BoardPosition {
    pub fn new(column: u8, row: u8) -> BoardPosition {
        BoardPosition::from_index(column + BitBoard::width() * row)
    }
    
    fn from_index(index: u8) -> BoardPosition {
        BoardPosition(BitBoard::from_uint(0x8000_0000_0000_0000_0000_0000_0000_0000 >> index))
    }
}

impl Display for BoardPosition {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        let index = self.0.leading_zeros();

        let y = index / BitBoard::width();

        let x = index - (BitBoard::width() * y);

        f.write_fmt(format_args!("({}, {})", x, y))
    }
}

/// A bitboard with 16 columns and 8 rows,
/// flowing left to right, then wrapping top to bottom.
#[derive(Copy, Clone)]
pub struct BitBoard(__m128i);

impl PartialEq for BitBoard {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            let neq = _mm_xor_si128(self.0, other.0);

            _mm_test_all_zeros(neq, neq) == 1
        }
    }
}

impl Eq for BitBoard {}

impl Debug for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let high_bits: u64 =
            unsafe { mem::transmute(_mm_cvtsi128_si64(_mm_unpackhi_epi64(self.0, self.0))) };
        let low_bits = unsafe { mem::transmute(_mm_cvtsi128_si64(self.0)) };

        for bits in &[high_bits, low_bits] {
            for i in 1..5 {
                f.write_str(&(format!("{:016b}", bits.rotate_left(i * 16) % 2u64.pow(16)) + "\n"))?;
            }
        }

        Ok(())
    }
}

impl BitAnd for BitBoard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        unsafe { BitBoard(_mm_and_si128(self.0, rhs.0)) }
    }
}

impl BitOr for BitBoard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        unsafe { BitBoard(_mm_or_si128(self.0, rhs.0)) }
    }
}

impl Not for BitBoard {
    type Output = BitBoard;

    fn not(self) -> BitBoard {
        unsafe { BitBoard(_mm_xor_si128(self.0, _mm_cmpeq_epi32(self.0, self.0))) }
    }
}

impl BitBoard {
    pub fn and_not(self, rhs: Self) -> Self {
        unsafe { BitBoard(_mm_andnot_si128(rhs.0, self.0)) }
    }

    pub fn nor(self, rhs: Self) -> Self {
        !(self | rhs)
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
        position.0
    }

    pub fn from_uint(int: u128) -> BitBoard {
        let items: [i64; 2] = unsafe { mem::transmute(int) };

        unsafe { BitBoard(_mm_set_epi64x(items[1], items[0])) }
    }

    pub fn top_edge() -> BitBoard {
        unsafe {
            BitBoard(_mm_setr_epi16(
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                mem::transmute(0xFFFFu16),
            ))
        }
    }

    pub fn bottom_edge() -> BitBoard {
        unsafe {
            BitBoard(_mm_setr_epi16(
                mem::transmute(0xFFFFu16),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ))
        }
    }

    pub fn right_edge() -> BitBoard {
        unsafe { BitBoard(_mm_set1_epi16(mem::transmute(0x0001u16))) }
    }

    pub fn left_edge() -> BitBoard {
        unsafe { BitBoard(_mm_set1_epi16(mem::transmute(0x8000u16))) }
    }

    pub fn empty() -> BitBoard {
        unsafe { BitBoard(_mm_setzero_si128()) }
    }

    pub fn shift_up(self) -> BitBoard {
        unsafe { BitBoard(_mm_bslli_si128(self.0, 2)) }
    }

    pub fn shift_down(self) -> BitBoard {
        unsafe { BitBoard(_mm_bsrli_si128(self.0, 2)) }
    }

    pub fn shift_left(self) -> BitBoard {
        unsafe { BitBoard(_mm_slli_epi16(self.0, 1)) }
    }

    pub fn shift_right(self) -> BitBoard {
        unsafe { BitBoard(_mm_srli_epi16(self.0, 1)) }
    }

    pub fn is_empty(&self) -> bool {
        unsafe { _mm_test_all_zeros(self.0, self.0) == 1 }
    }

    pub fn is_set(self, position: BoardPosition) -> bool {
        !(self & Self::singleton(position)).is_empty()
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
        self.and_not(self.interior())
    }

    pub fn immediate_exterior(self) -> BitBoard {
        self.expand_one().and_not(self)
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

    pub fn first_cell_position(self) -> BoardPosition {
        BoardPosition::from_index(self.leading_zeros())
    }

    fn leading_zeros(self) -> u8 {
        let high_bits = unsafe { _mm_cvtsi128_si64(_mm_unpackhi_epi64(self.0, self.0)) };

        let leading_zeros = if high_bits != 0 {
            high_bits.leading_zeros()
        } else {
            unsafe { _mm_cvtsi128_si64(self.0).leading_zeros() + 64 }
        };

        leading_zeros as u8
    }

    pub fn first_cell_board(self) -> BitBoard {
        BitBoard::singleton(self.first_cell_position())
    }

    // Gets all single points on the board
    pub fn singletons(self) -> BitBoard {
        self.and_not(self.shift_up())
            .and_not(self.shift_down())
            .and_not(self.shift_left())
            .and_not(self.shift_right())
    }

    pub fn count(self) -> u32 {
        unsafe {
            _mm_cvtsi128_si64(_mm_unpackhi_epi64(self.0, self.0)).count_ones()
                + _mm_cvtsi128_si64(self.0).count_ones()
        }
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
            let first_group = self
                .remaining_groups
                .first_cell_board()
                .flood_fill(self.remaining_groups);

            self.remaining_groups = self.remaining_groups.and_not(first_group);

            Some(first_group)
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
            let position = self.remaining_positions.first_cell_position();

            self.remaining_positions = self
                .remaining_positions
                .and_not(BitBoard::singleton(position));

            Some(position)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

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

    #[test]
    fn first_cell_board() {
        let board = BitBoard::from_uint(0b0000000000000000_0101011000000000_0000000100100000_0001011001101000_0000100000100000_0001000001110000_0000000000000000_0000000000000000);

        assert_eq!(
            format!("{:?}", board.first_cell_board()),
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

    #[test]
    fn first_cell2() {
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

        assert_eq!(
            format!("{:?}", board.first_cell_board()),
            "0000000000000000\n\
             1000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n"
        );
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
    fn first_cell_position() {
        let board = BitBoard::from_uint(0b1000000000000000_0110000000000000_0000000000010000_0000000000000000_0000000000000000_0000000000000000_1000000000000000_1100000000000000);

        assert_eq!(board.first_cell_position(), BoardPosition::new(0, 0));

        let board = BitBoard::from_uint(0b0000000000000000_0110000000000000_0000000000010000_0000000000000000_0000000000000000_0000000000000000_1000000000000000_1100000000000000);

        assert_eq!(board.first_cell_position(), BoardPosition::new(1, 1));

        let board = BitBoard::from_uint(0b0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_1000000000000000_1100000000000000);

        assert_eq!(board.first_cell_position(), BoardPosition::new(0, 6));
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
}

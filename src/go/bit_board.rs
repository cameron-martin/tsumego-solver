use std::fmt;
use std::fmt::Debug;
use std::ops::{BitAnd, BitOr, Not};

use super::BoardCoord;

pub const BOARD_WIDTH: usize = 16;
pub const BOARD_HEIGHT: usize = 8;

/// A bitboard with 16 columns and 8 rows,
/// flowing left to right, then wrapping top to bottom.
#[derive(Copy, Clone, PartialEq)]
pub struct BitBoard(u128);

impl Debug for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in 0..BOARD_HEIGHT {
            let row = (self.0 << (i * BOARD_WIDTH)) >> ((BOARD_HEIGHT - 1) * BOARD_WIDTH);
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

impl BitBoard {
    pub fn singleton(position: BoardCoord) -> BitBoard {
        BitBoard(0x80000000000000000000000000000000 >> (position.0 + (BOARD_WIDTH * position.1)))
    }

    pub fn empty() -> BitBoard {
        BitBoard(0)
    }

    pub fn shift_up(self) -> BitBoard {
        BitBoard(self.0 << BOARD_WIDTH)
    }

    pub fn shift_down(self) -> BitBoard {
        BitBoard(self.0 >> BOARD_WIDTH)
    }

    pub fn shift_left(self) -> BitBoard {
        BitBoard((self.0 << 1) & 0xFFFEFFFEFFFEFFFEFFFEFFFEFFFEFFFEu128)
    }

    pub fn shift_right(self) -> BitBoard {
        BitBoard((self.0 >> 1) & 0x7FFF7FFF7FFF7FFF7FFF7FFF7FFF7FFFu128)
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
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
        self & self.shift_up() & self.shift_down() & self.shift_left() & self.shift_right()
    }

    pub fn border(self) -> BitBoard {
        self & !self.interior()
    }

    pub fn immediate_exterior(self) -> BitBoard {
        self.expand_one() & !self
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn flood_fill() {
        let mask = BitBoard(0b0000000000000000_0101011000000000_0000000100100000_0001011001101000_0000100000100000_0001000001110000_0000000000000000_0000000000000000);

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

        let filled = BitBoard::singleton((11, 5)).flood_fill(mask);

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
    fn shift_right() {
        let board = BitBoard(0b0000000000000000_0000000000000000_0010000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000);

        assert_eq!(
            format!("{:?}", board.shift_right()),
            "0000000000000000\n\
             0000000000000000\n\
             0001000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n\
             0000000000000000\n"
        );
    }

    #[test]
    fn shift_right_on_edge() {
        let board = BitBoard(0b0000000000000000_0000000000000000_0000000000000001_0000000000000000_0000000000000000_0000000000000000_0000000000000000_0000000000000000);

        assert!(board.shift_right().is_empty());
    }
}

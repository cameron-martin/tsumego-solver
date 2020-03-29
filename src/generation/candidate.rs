use crate::go::{BitBoard, BoardPosition, GoBoard, GoPlayer};
use rand::prelude::*;

pub fn generate_candidate() -> GoBoard {
    let mut rng = thread_rng();

    let width: u8 = rng.gen_range(2, BitBoard::width() - 1);
    let height: u8 = rng.gen_range(2, BitBoard::height() - 1);

    let enclosure = generate_enclosure(width, height);

    let surround = enclosure.immediate_exterior();
    let out_of_bounds = !(enclosure | surround);

    let (mut black, mut white) = generate_interior_stones(width, height);

    let attacker = if random() {
        GoPlayer::White
    } else {
        GoPlayer::Black
    };

    match attacker {
        GoPlayer::White => white = white | surround,
        GoPlayer::Black => black = black | surround,
    };

    GoBoard::new(black, white, out_of_bounds)
}

fn generate_enclosure(width: u8, height: u8) -> BitBoard {
    let mut board = BitBoard::empty();

    for i in 0..width {
        for j in 0..height {
            board = board.set(BoardPosition::new(i, j));
        }
    }

    board
}

fn generate_interior_stones(width: u8, height: u8) -> (BitBoard, BitBoard) {
    let mut black = BitBoard::empty();
    let mut white = BitBoard::empty();

    for i in 0..width {
        for j in 0..height {
            let is_filled: bool = random();

            if is_filled {
                let position = BoardPosition::new(i, j);
                if random() {
                    white = white.set(position);
                } else {
                    black = black.set(position);
                }
            }
        }
    }

    (black, white)
}

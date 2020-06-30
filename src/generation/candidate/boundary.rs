use crate::go::{BitBoard, BoardPosition};
use rand::prelude::*;

pub fn generate_in_bounds<G: Rng>(rng: &mut G) -> BitBoard {
    let width: u8 = rng.gen_range(2, BitBoard::width() - 1);
    let height: u8 = rng.gen_range(2, BitBoard::height() - 1);

    let mut board = BitBoard::empty();

    for i in 0..width {
        for j in 0..height {
            board = board.set(BoardPosition::new(i, j));
        }
    }

    board
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::go::GoBoard;
    use insta::assert_snapshot;
    use std::iter;

    #[test]
    fn snapshot_generate_in_bounds() {
        let mut snapshot = String::new();

        let mut rng = StdRng::seed_from_u64(0);

        let boundaries = iter::repeat_with(|| generate_in_bounds(&mut rng));

        for in_bounds in boundaries.take(100) {
            let surround = in_bounds.immediate_exterior();
            let board = GoBoard::new(surround, BitBoard::empty(), !(in_bounds | surround));

            snapshot.push_str(&format!("{}\n\n", board));
        }

        assert_snapshot!(snapshot);
    }
}

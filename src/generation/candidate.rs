mod boundary;

use crate::go::{BitBoard, GoBoard, GoPlayer};
use rand::prelude::*;

pub fn generate_candidate<G: Rng>(rng: &mut G) -> GoBoard {
    let in_bounds = boundary::generate_in_bounds(rng);

    let surround = in_bounds.immediate_exterior();
    let out_of_bounds = !(in_bounds | surround);

    let (mut black, mut white) = generate_interior_stones(in_bounds, rng);

    let attacker = if rng.gen() {
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

fn generate_interior_stones<G: RngCore>(in_bounds: BitBoard, rng: &mut G) -> (BitBoard, BitBoard) {
    let mut black = BitBoard::empty();
    let mut white = BitBoard::empty();

    for position in in_bounds.positions() {
        let is_filled: bool = rng.gen();

        if is_filled {
            if rng.gen() {
                white = white.set(position);
            } else {
                black = black.set(position);
            }
        }
    }

    (black, white)
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;
    use std::iter;

    #[test]
    fn snapshot_generate_candidate() {
        let mut snapshot = String::new();

        let mut rng = StdRng::seed_from_u64(0);

        let candidates = iter::repeat_with(|| generate_candidate(&mut rng));

        for candidate in candidates.take(100) {
            snapshot.push_str(&format!("{}\n\n", candidate));
        }

        assert_snapshot!(snapshot);
    }
}

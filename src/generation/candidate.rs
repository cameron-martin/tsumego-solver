mod boundary;

use crate::go::{BitBoard, GoBoard, GoPlayer};
use rand::prelude::*;

pub fn generate_candidate<G: Rng>(rng: &mut G) -> GoBoard {
    let playable_area = boundary::generate_playable_area(rng);

    let boundary = boundary::draw_boundary(playable_area);
    let out_of_bounds = !(playable_area | boundary);

    let (mut black, mut white) = generate_interior_stones(playable_area, rng);

    let attacker = if rng.gen() {
        GoPlayer::White
    } else {
        GoPlayer::Black
    };

    match attacker {
        GoPlayer::White => white = white | boundary,
        GoPlayer::Black => black = black | boundary,
    };

    GoBoard::new(black, white, out_of_bounds)
}

fn generate_interior_stones<G: RngCore>(
    playable_area: BitBoard,
    rng: &mut G,
) -> (BitBoard, BitBoard) {
    let mut black = BitBoard::empty();
    let mut white = BitBoard::empty();

    for position in playable_area.positions() {
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

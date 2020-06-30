use crate::go::{BitBoard, BitBoardEdge, BoardPosition};
use rand::distributions::weighted::WeightedIndex;
use rand::prelude::*;

pub fn generate_playable_area<G: Rng>(rng: &mut G) -> BitBoard {
    let starting_position = BoardPosition::new(
        rng.gen_range(0, BitBoard::width()),
        rng.gen_range(0, BitBoard::height()),
    );

    let iterations: usize = rng.gen_range(10, 30);

    let mut playable_area = BitBoard::singleton(starting_position);

    for _ in 0..iterations {
        let mut candidates = Vec::new();
        let mut weights = Vec::new();

        for &edge in BitBoardEdge::iter() {
            let mut candidate_next_positions = playable_area.shift_towards(edge) & !playable_area;
            if !(playable_area & BitBoard::edge(edge.opposite())).is_empty() {
                candidate_next_positions = candidate_next_positions & !(BitBoard::edge(edge));
            }

            candidates.push(candidate_next_positions);
            weights.push(candidate_next_positions.count());
        }

        let dist = WeightedIndex::new(&weights).unwrap();

        let candidate_next_positions = candidates[dist.sample(rng)];

        let next_position = candidate_next_positions
            .positions()
            .nth(rng.gen_range(0, candidate_next_positions.count()) as usize)
            .unwrap();

        playable_area = playable_area.set(next_position);
    }

    let unplayable_area = (!playable_area)
        .groups()
        .max_by_key(|group| group.count())
        .unwrap();

    !unplayable_area
}

pub fn draw_boundary(playable_area: BitBoard) -> BitBoard {
    (playable_area.expand_one()
        | playable_area.shift_right().shift_up()
        | playable_area.shift_right().shift_down()
        | playable_area.shift_left().shift_up()
        | playable_area.shift_left().shift_down())
        & !playable_area
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::go::GoBoard;
    use insta::assert_snapshot;
    use std::iter;

    use quickcheck::{Arbitrary, Gen};
    use quickcheck_macros::quickcheck;

    #[test]
    fn snapshot_generate_playable_area() {
        let mut snapshot = String::new();

        let mut rng = StdRng::seed_from_u64(0);

        let playable_areas = iter::repeat_with(|| generate_playable_area(&mut rng));

        for playable_area in playable_areas.take(100) {
            let boundary = draw_boundary(playable_area);
            let board = GoBoard::new(boundary, BitBoard::empty(), !(playable_area | boundary));

            snapshot.push_str(&format!("{}\n\n", board));
        }

        assert_snapshot!(snapshot);
    }

    #[derive(Clone, Debug)]
    struct InBounds(BitBoard);

    impl Arbitrary for InBounds {
        fn arbitrary<G: Gen>(g: &mut G) -> InBounds {
            InBounds(generate_playable_area(g))
        }
    }

    /// Otherwise the illusion that the puzzles are actually a part of a regular sized board is broken
    #[quickcheck]
    fn boundary_does_not_extend_to_opposite_edges(board: InBounds) {
        assert!(
            (BitBoard::top_edge() & board.0).is_empty()
                || (BitBoard::bottom_edge() & board.0).is_empty()
        );
        assert!(
            (BitBoard::left_edge() & board.0).is_empty()
                || (BitBoard::right_edge() & board.0).is_empty()
        );
    }
}

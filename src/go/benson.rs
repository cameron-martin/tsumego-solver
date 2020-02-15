use super::{BitBoard, GoBoard, GoPlayer};

impl GoBoard {
    fn small_x_enclosed_regions(&self, x: GoPlayer) -> BitBoard {
        let regions = !self.get_bitboard_for_player(x);
        let regions_with_empty_interiors =
            (regions.interior() & self.empty_cells()).flood_fill(regions);

        regions & !regions_with_empty_interiors
    }

    pub fn unconditionally_alive_blocks_for_player(&self, player: GoPlayer) -> BitBoard {
        let mut regions = self.small_x_enclosed_regions(player);
        let blocks = self.get_bitboard_for_player(player);
        let mut remaining_blocks = blocks;

        loop {
            let mut removed_blocks = 0;

            for block in remaining_blocks.iter() {
                // Has the block `block` got at least 2 healthy regions in `regions`?

                let empty_intersections_in_regions = regions & self.empty_cells();

                let unhealthy_regions = (empty_intersections_in_regions
                    & !block.immediate_exterior())
                .flood_fill(regions);

                let healthy_regions = regions & !unhealthy_regions;

                let more_than_one_healthy_region = !healthy_regions.is_empty()
                    && !(healthy_regions
                        & !healthy_regions.first_cell().flood_fill(healthy_regions))
                    .is_empty();

                if !more_than_one_healthy_region {
                    remaining_blocks = remaining_blocks & !block;

                    removed_blocks += 1;
                }
            }

            if removed_blocks == 0 {
                return remaining_blocks;
            }

            let removed_blocks = blocks & !remaining_blocks;
            regions = regions & !(removed_blocks.expand_one() & regions).flood_fill(regions);
        }
    }

    pub fn unconditionally_alive_blocks(&self) -> GoBoard {
        GoBoard {
            white: self.unconditionally_alive_blocks_for_player(GoPlayer::White),
            black: self.unconditionally_alive_blocks_for_player(GoPlayer::Black),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::GoGame;
    use super::*;

    #[test]
    fn small_black_enclosed_regions() {
        let board = GoGame::from_sgf(include_str!(
            "../test_sgfs/small_black_enclosed_regions.sgf"
        ))
        .get_board();

        let answer = GoGame::from_sgf(include_str!(
            "../test_sgfs/small_black_enclosed_regions_answer.sgf"
        ))
        .get_board()
        .get_bitboard_for_player(GoPlayer::White);

        assert_eq!(board.small_x_enclosed_regions(GoPlayer::Black), answer);
    }

    #[test]
    fn all_alive1() {
        let game = GoGame::from_sgf(include_str!("../test_sgfs/life_and_death/all_alive1.sgf"));

        assert_eq!(
            game.get_board()
                .unconditionally_alive_blocks_for_player(GoPlayer::White),
            game.get_board().get_bitboard_for_player(GoPlayer::White)
        );
    }

    #[test]
    fn all_dead1() {
        let game = GoGame::from_sgf(include_str!("../test_sgfs/life_and_death/all_dead1.sgf"));

        assert_eq!(
            game.get_board().unconditionally_alive_blocks(),
            GoBoard::empty(),
        );
    }

    #[test]
    fn mixture() {
        let game = GoGame::from_sgf(include_str!("../test_sgfs/life_and_death/mixture.sgf"));
        let answer = GoGame::from_sgf(include_str!(
            "../test_sgfs/life_and_death/mixture_answer.sgf"
        ));

        assert_eq!(
            game.get_board().unconditionally_alive_blocks(),
            *answer.get_board(),
        );
    }
}

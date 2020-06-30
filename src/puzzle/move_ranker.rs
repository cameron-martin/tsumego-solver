use crate::go::{GoGame, GoPlayer, Move, MovesIncPassIterator};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::path::Path;
use tensorflow::{Graph, Session, SessionOptions, SessionRunArgs, Tensor};

pub trait MoveRanker {
    type Iter: Iterator<Item = (GoGame, Move)>;

    fn order_moves(&self, game: GoGame) -> Self::Iter;
}

pub struct LinearMoveRanker;

impl MoveRanker for LinearMoveRanker {
    type Iter = MovesIncPassIterator;

    fn order_moves(&self, game: GoGame) -> Self::Iter {
        game.generate_moves_including_pass()
    }
}

pub struct CnnMoveRanker {
    graph: Graph,
    session: Session,
}

impl CnnMoveRanker {
    pub fn new(model_dir: &Path) -> Self {
        let mut graph = Graph::new();

        let session =
            Session::from_saved_model(&SessionOptions::new(), &["serve"], &mut graph, model_dir)
                .unwrap();

        Self { graph, session }
    }
}

pub struct CnnMoveIterator {
    moves: Vec<(GoGame, Move)>,
}

impl Iterator for CnnMoveIterator {
    type Item = (GoGame, Move);
    fn next(&mut self) -> Option<Self::Item> {
        self.moves.pop()
    }
}

impl MoveRanker for CnnMoveRanker {
    type Iter = CnnMoveIterator;

    fn order_moves(&self, game: GoGame) -> Self::Iter {
        let child_moves: Vec<_> = game.generate_moves_including_pass().collect();

        let mut input_tensor = Tensor::<f32>::new(&[child_moves.len() as u64, 8, 16, 3]);

        let mut i = 0;
        for (child_game, _go_move) in child_moves.iter() {
            let board = if child_game.current_player == GoPlayer::Black {
                child_game.board
            } else {
                child_game.board.invert_colours()
            };

            let black = board.get_bitboard_for_player(GoPlayer::Black).to_uint();
            let white = board.get_bitboard_for_player(GoPlayer::White).to_uint();
            let in_bounds = (!board.out_of_bounds()).to_uint();

            let mut mask: u128 = 1 << 127;
            for j in 0..8 {
                for k in 0..16 {
                    input_tensor.set(&[i, j, k, 0], if black & mask != 0 { 1.0 } else { 0.0 });
                    input_tensor.set(&[i, j, k, 1], if white & mask != 0 { 1.0 } else { 0.0 });
                    input_tensor.set(&[i, j, k, 2], if in_bounds & mask != 0 { 1.0 } else { 0.0 });
                    mask = mask >> 1;
                }
            }

            i += 1;
        }

        // These were gathered using the following command:
        // saved_model_cli show --dir model --tag_set serve --signature_def serving_default
        // See https://medium.com/analytics-vidhya/deploying-tensorflow-2-1-as-c-c-executable-1d090845055c
        let input = self
            .graph
            .operation_by_name_required("serving_default_input_1")
            .unwrap();
        let output = self
            .graph
            .operation_by_name_required("StatefulPartitionedCall")
            .unwrap();

        let mut args = SessionRunArgs::new();

        args.add_feed(&input, 0, &input_tensor);
        let result_token = args.request_fetch(&output, 0);

        self.session.run(&mut args).unwrap();

        let result_tensor = args.fetch::<f32>(result_token).unwrap();

        let mut iter_child_moves = child_moves.iter().enumerate().collect::<Vec<_>>();

        iter_child_moves.sort_by(|(i1, _), (i2, _)| {
            let t1 = result_tensor.get(&[*i1 as u64, 0]);
            let t2 = result_tensor.get(&[*i2 as u64, 0]);

            t2.partial_cmp(&t1).unwrap()

            // match game.current_player {
            //     GoPlayer::White => t1.partial_cmp(&t2).unwrap(),
            //     GoPlayer::Black => t2.partial_cmp(&t1).unwrap(),
            // }
        });

        CnnMoveIterator {
            moves: iter_child_moves
                .iter()
                .map(|&(_, result)| *result)
                .collect(),
        }
    }
}

pub struct OrderedMovesIterator {
    game: GoGame,
    remaining_moves: Vec<Move>,
}

impl Iterator for OrderedMovesIterator {
    type Item = (GoGame, Move);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(go_move) = self.remaining_moves.pop() {
                if let Ok(new_game) = self.game.play_move(go_move) {
                    return Some((new_game, go_move));
                }
            } else {
                return None;
            }
        }
    }
}

pub struct RandomMoveRanker;

impl MoveRanker for RandomMoveRanker {
    type Iter = OrderedMovesIterator;

    fn order_moves(&self, game: GoGame) -> Self::Iter {
        let mut moves = Vec::new();

        for position in game.board.playable_cells().positions() {
            moves.push(Move::Place(position));
        }

        moves.push(Move::Pass);

        moves.shuffle(&mut thread_rng());

        OrderedMovesIterator {
            game,
            remaining_moves: moves,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CnnMoveRanker, MoveRanker};
    use crate::go::{BoardPosition, GoGame, GoPlayer, Move};
    use std::path::Path;

    #[test]
    fn move_ordering_ultrasimple1() {
        let game = GoGame::from_sgf(
            include_str!("../test_sgfs/puzzles/true_ultrasimple1.sgf"),
            GoPlayer::Black,
        );

        let ranker = CnnMoveRanker::new(Path::new("network/model"));
        let mut moves = ranker.order_moves(game);

        assert_eq!(
            moves.next().unwrap().1,
            Move::Place(BoardPosition::new(1, 0))
        );
    }

    #[test]
    fn move_ordering_ultrasimple2() {
        let game = GoGame::from_sgf(
            include_str!("../test_sgfs/puzzles/true_ultrasimple2.sgf"),
            GoPlayer::Black,
        );

        let ranker = CnnMoveRanker::new(Path::new("network/model"));
        let mut moves = ranker.order_moves(game);

        assert_eq!(
            moves.next().unwrap().1,
            Move::Place(BoardPosition::new(1, 0))
        );
    }

    #[test]
    fn move_ordering_w_simple2() {
        let game = GoGame::from_sgf(
            include_str!("../test_sgfs/puzzles/true_simple2.sgf"),
            GoPlayer::Black,
        );
        let game = GoGame::from_board(game.board.invert_colours(), GoPlayer::White);
        let ranker = CnnMoveRanker::new(Path::new("network/model"));
        let mut moves = ranker.order_moves(game);

        assert_eq!(
            moves.next().unwrap().1,
            Move::Place(BoardPosition::new(2, 1))
        );
    }

    #[test]
    fn move_ordering_w2_simple2() {
        let game = GoGame::from_sgf(
            include_str!("../test_sgfs/puzzles/true_simple2.sgf"),
            GoPlayer::White,
        );
        let ranker = CnnMoveRanker::new(Path::new("network/model"));
        let mut moves = ranker.order_moves(game);

        assert_eq!(
            moves.next().unwrap().1,
            Move::Place(BoardPosition::new(2, 1))
        );
    }

    #[test]
    fn move_ordering_b_simple2() {
        let game = GoGame::from_sgf(
            include_str!("../test_sgfs/puzzles/true_simple2.sgf"),
            GoPlayer::Black,
        );

        let ranker = CnnMoveRanker::new(Path::new("network/model"));
        let mut moves = ranker.order_moves(game);

        assert_eq!(
            moves.next().unwrap().1,
            Move::Place(BoardPosition::new(2, 1))
        );
    }
}

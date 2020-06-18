use crate::go::{BoardPosition, GoBoard, GoGame, GoPlayer, Move};
use std::path::Path;
use tensorflow::{Graph, Session, SessionOptions, SessionRunArgs, Tensor};

pub struct MoveRanker {
    graph: Graph,
    session: Session,
}

impl MoveRanker {
    pub fn new(model_dir: &Path) -> MoveRanker {
        let mut graph = Graph::new();

        let session =
            Session::from_saved_model(&SessionOptions::new(), &["serve"], &mut graph, model_dir)
                .unwrap();

        MoveRanker { graph, session }
    }

    pub fn order_moves(&self, game: GoGame) -> Vec<Move> {
        let board = if game.current_player == GoPlayer::Black {
            game.board
        } else {
            game.board.invert_colours()
        };

        let mut input_tensor = Tensor::<f32>::new(&[1, 8, 16, 3]);

        let black = board.get_bitboard_for_player(GoPlayer::Black).to_uint();
        let white = board.get_bitboard_for_player(GoPlayer::White).to_uint();
        let in_bounds = (!board.out_of_bounds()).to_uint();

        let mut mask: u128 = 1 << 127;
        for i in 0..8 {
            for j in 0..16 {
                input_tensor.set(&[0, i, j, 0], if black & mask != 0 { 1.0 } else { 0.0 });
                input_tensor.set(&[0, i, j, 1], if white & mask != 0 { 1.0 } else { 0.0 });
                input_tensor.set(&[0, i, j, 2], if in_bounds & mask != 0 { 1.0 } else { 0.0 });
                mask = mask >> 1;
            }
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

        let mut result_vec = Vec::with_capacity(129);

        for i in 0..128 {
            result_vec.push((
                result_tensor.get(&[0, i]),
                Move::Place(BoardPosition(i as u8)),
            ));
        }

        result_vec.push((result_tensor.get(&[0, 128]), Move::Pass));

        result_vec.sort_by(|(weight1, _), (weight2, _)| weight1.partial_cmp(weight2).unwrap());

        result_vec
            .iter()
            .map(|(weight, go_move)| *go_move)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::MoveRanker;
    use crate::go::{BoardPosition, GoGame, Move};
    use std::path::Path;

    #[test]
    fn move_ordering_ultrasimple1() {
        let game = GoGame::from_sgf(include_str!("../test_sgfs/puzzles/true_ultrasimple1.sgf"));

        let ranker = MoveRanker::new(Path::new("network/model"));
        let moves = ranker.order_moves(game);

        assert_eq!(
            *moves.last().unwrap(),
            Move::Place(BoardPosition::new(1, 0))
        );
    }

    #[test]
    fn move_ordering_ultrasimple2() {
        let game = GoGame::from_sgf(include_str!("../test_sgfs/puzzles/true_ultrasimple2.sgf"));

        let ranker = MoveRanker::new(Path::new("network/model"));
        let moves = ranker.order_moves(game);

        assert_eq!(
            *moves.last().unwrap(),
            Move::Place(BoardPosition::new(0, 0))
        );
    }
}

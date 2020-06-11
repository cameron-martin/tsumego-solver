use crate::go::{GoBoard, GoGame, GoPlayer, PassState};

pub fn is_terminal(game: GoGame, player: GoPlayer, attacker: GoPlayer) -> Option<bool> {
    let defender = attacker.flip();

    // If both players pass sequentially, the game ends and
    // the player to pass second loses.
    if game.pass_state == PassState::PassedTwice {
        Some(game.current_player == player)
    // If the defender has unconditionally alive blocks, the defender wins
    } else if !game
        .board
        .unconditionally_alive_blocks_for_player(defender)
        .is_empty()
    {
        Some(defender == player)
    // If the defender doesn't have any space to create eyes, the attacker wins
    } else if !can_defender_live(game.board, attacker) {
        Some(attacker == player)
    // Otherwise, the result is a non-terminal node
    } else {
        None
    }
}

/// Whether it's possible for the defender to live.
/// It's possible if there are at least two-non-adjacent interior points
/// in the area not occupied by safe stones.
fn can_defender_live(board: GoBoard, attacker: GoPlayer) -> bool {
    let safe_attacker_stones = board
        .out_of_bounds()
        .expand_one()
        .flood_fill(board.get_bitboard_for_player(attacker));

    let maximum_living_shape = !safe_attacker_stones & !board.out_of_bounds();

    let interior = maximum_living_shape.interior();
    let interior_count = interior.count();

    interior_count > 2 || (interior_count == 2 && !interior.singletons().is_empty())
}

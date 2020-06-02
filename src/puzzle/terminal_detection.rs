use crate::go::{GoBoard, GoGame, GoPlayer, PassState};

pub fn is_terminal(game: GoGame, player: GoPlayer, attacker: GoPlayer) -> Option<bool> {
    let defender = attacker.flip();

    // If both players pass sequentially, the game ends and
    // the player to pass second loses.
    if game.pass_state == PassState::PassedTwice {
        Some(game.current_player == player)
    // If the defender has unconditionally alive blocks, the defender wins
    } else if !game
        .get_board()
        .unconditionally_alive_blocks_for_player(defender)
        .is_empty()
    {
        Some(defender == player)
    // If the defender doesn't have any space to create eyes, the attacker wins
    } else if is_defender_dead(game.get_board(), attacker) {
        Some(attacker == player)
    // Otherwise, the result is a non-terminal node
    } else {
        None
    }
}

/// A conservative estimate on whether the group is dead.
/// true means it's definitely dead, false otherwise
fn is_defender_dead(board: GoBoard, attacker: GoPlayer) -> bool {
    let attacker_alive = board
        .out_of_bounds()
        .expand_one()
        .flood_fill(board.get_bitboard_for_player(attacker));

    let maximum_living_shape = !attacker_alive & !board.out_of_bounds();

    maximum_living_shape.interior().count() < 2
}

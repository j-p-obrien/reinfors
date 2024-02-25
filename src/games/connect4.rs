/// An implementation of Connect 4.
///
/// This game is solved, and we know that Player 1 has a winning strategy. Using a minimax strategy
/// should always guarantee a win for Player 1. Sorry Player 2!
use crate::game_state::Player;

pub enum PlayerColor {
    Red,
    Black,
    Empty,
}

#[allow(dead_code)]
pub struct Connect4 {
    board: [[u8; 6]; 2],
    current_player: Player,
    player1_color: PlayerColor,
}
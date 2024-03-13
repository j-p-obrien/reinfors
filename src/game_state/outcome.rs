use super::*;

/// The type of a Game outcome where there either is one definite winner or a draw. This is for
/// Games like chess, checkers, tic-tac-toe, Monopoly etc.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum WinDraw<G>
where
    G: GameState,
{
    Win(G::Player),
    Draw,
}

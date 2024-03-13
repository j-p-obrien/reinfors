use super::*;

pub type GameResult<T, G> = Result<T, GameError<G>>;

/// Potential errors in the playing of the game.
#[derive(Debug)]
pub enum GameError<G>
where
    G: GameState,
{
    /// Should be returned if the user tries to apply an Action that is illegal in the current
    /// DynamicGameState.
    /// Returns the associated DynamicGameState and Action.
    IllegalAction(G, G::Action),
    /// Should be returned if the user tries to apply an action to a Game that is already over or
    /// tries to call a safe Strategy when the Game is over.
    /// Returns the associated DynamicGameState.
    GameOver(G),
    /// Should be returned if the Game isn't over but there are no legal actions.
    /// Returns the DynamicGameState this occured in.
    NoLegalActions(G),
    /// Should be returned if the strategy can't pick a move but the Game is not yet over.
    /// Returns the DynamicGameState this occured in.
    StrategyFailure(G),
    /// Should be returned if the Evaluator fails somehow
    EvaluatorFailure(G, Vec<G::Action>),
    /// For arbitrary errors.
    Arbitrary(&'static str),
}

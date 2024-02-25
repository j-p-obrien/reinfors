use std::cmp::Ordering;
use std::hash::Hash;

use crate::evaluator::*;
use crate::game_state::*;

/// The trait for strategies. Given a DynamicGameState, return either an Action or a GameError.
/// Strategies can use the output of the Evaluator in very different ways. For instance, you may
/// have an evaluator that returns a policy. From here, what should you do with the given probabilities?
/// You could simply pick the action with the highest probability, or you could do a Monte-Carlo
/// tree-search type algorithm where you randomly sample from the given probabilities and
/// potentially recurse through the Game's states with the sampled actions.
///
/// Generally, provided strategies assume that the game is not over. This avoids a redundant call
/// to DynamicGameState::outcome() while inside best_action(). The provided GamePlayers all call
/// this function before calling best_action(), and thus the code in this module reflects that.
pub trait Strategy<G, E>
where
    G: DynamicGameState,
    E: Evaluator<G>,
{
    fn best_action(&mut self, state: &G, evaluator: &mut E) -> GameResult<G::Action, G>;
}

/// Takes an Evaluator whose Evaluation impl's PartialOrd and returns the action that has the highest
/// Evaluation. If there are no legal actions, we return Err(GameError::NoLegalActions(current GameState)).
/// If any evaluations result in an Err, then this strategy returns that Err. If
/// any of the partial comparisons result in None, then we return
/// Err(GameError::EvaluatorFailure(current GameState, vec![action1, action2])).
pub struct GreedyStrategy;

impl<G, E> Strategy<G, E> for GreedyStrategy
where
    G: GameState + Clone,
    G::Action: 'static + Clone,
    E: Evaluator<G>,
    E::Evaluation: PartialOrd,
{
    fn best_action(&mut self, state: &G, evaluator: &mut E) -> GameResult<G::Action, G> {
        let mut actions = state.legal_actions();
        let mut accum = if let Some(action) = actions.next() {
            let eval = evaluator.evaluate(state, action)?;
            (action, eval)
        } else {
            return Err(GameError::NoLegalActions(state.clone()));
        };
        for action in actions {
            let eval = evaluator.evaluate(state, action)?;
            accum = match accum.1.partial_cmp(&eval) {
                Some(ordering) => match ordering {
                    Ordering::Less => (action, eval),
                    _ => accum,
                },
                None => {
                    return Err(GameError::EvaluatorFailure(
                        state.clone(),
                        vec![accum.0.clone(), action.clone()],
                    ))
                }
            };
        }
        todo!()
    }
}

#[derive(Debug)]
pub struct MinMax;

impl<G> Strategy<G, EndStateEvaluator<G>> for MinMax
where
    G: GameState<Outcome = WinDrawOutcome<G>> + Hash + Eq + Clone,
    G::Action: 'static + Clone,
    G::Player: Eq,
    G::Outcome: Clone,
{
    fn best_action(
        &mut self,
        state: &G,
        evaluator: &mut EndStateEvaluator<G>,
    ) -> GameResult<G::Action, G> {
        let mut actions = state.legal_actions().peekable();
        // If there are no legal Actions, say so.
        if actions.peek().is_none() {
            return Err(GameError::NoLegalActions(state.clone()));
        }
        // If there is a winning Action, return it, but also collect a potential draw and losing
        // Action as well.
        let mut draw = None;
        let mut loser = None;
        for action in actions {
            let eval = evaluator.evaluate(state, action)?;
            match eval {
                WinDrawOutcome::Win(player) if player == state.current_player() => {
                    return Ok(action.clone());
                }
                WinDrawOutcome::Draw => draw = Some(action),
                _ => loser = Some(action),
            };
        }
        // No winning moves. Take draw if we can.
        if let Some(action) = draw {
            return Ok(action.clone());
        }
        // Uh oh, no draws available. We must accept defeat.
        // TODO: Maybe add the ability to offer a draw.
        if let Some(action) = loser {
            Ok(action.clone())
        } else {
            // If this is reached it means there were available actions but none were winners,
            // losers, or draws. So what happened?
            Err(GameError::StrategyFailure(state.clone()))
        }
    }
}

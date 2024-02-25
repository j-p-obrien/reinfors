use std::{collections::HashMap, hash::Hash};

use crate::game_state::*;

pub trait Evaluator<G>
where
    G: DynamicGameState,
{
    type Evaluation;

    fn evaluate(&mut self, state: &G, action: &G::Action) -> GameResult<Self::Evaluation, G>;
}

#[derive(Debug)]
pub struct RandomEvaluator {
    rng_state: u64,
}

impl RandomEvaluator {
    pub fn new(seed: u64) -> Self {
        Self { rng_state: seed }
    }
}

impl<G> Evaluator<G> for RandomEvaluator
where
    G: DynamicGameState,
{
    type Evaluation = u64;

    fn evaluate(&mut self, _state: &G, _action: &G::Action) -> GameResult<Self::Evaluation, G> {
        const A: u64 = 1664525;
        const C: u64 = 1013904223;

        self.rng_state = self.rng_state.wrapping_mul(A).wrapping_add(C);
        Ok(self.rng_state)
    }
}

/// This evaluator recurses through the legal actions available at each stage of the game and thus
/// MAY BE VERY EXPENSIVE TO COMPUTE!!! This evaluator is completely infeasible to compute for
/// anything more than very simple games.
///
/// Given a state and an action, the evaluator computes the resulting new state. If this new state
/// has been visited before, the outcome that has already been computed is returned. Otherwise, if
/// a call to DynamicGameState::outcome()
/// results in Some(Outcome), it will cache that outcome and return it. Otherwise, the current value of the new
/// state is (currently) indeterminate. The evaluator will then look at the legal options available
/// to the player in the new state and return the outcome in the following priority:
/// 1. If there are no legal actions available at this point, and the call to DynamicGameState::outcome()
/// returned None, then there is a logic error in the implementation of the game. If this happens,
/// then we return GameError::NoLegalMoves(current GameState).
/// 2. If any actions result in a win for the Player in the new state, WinDrawOutcome(Win(current player))
/// is returned.
/// 2. Otherwise, if there is a draw available to the player in the new state, WinDrawOutcome::Draw
/// is returned.
/// 3. Otherwise, there are only losing moves available for the current player. Ideally, we never reach
/// this part of the function because this would be identified earlier when we called DynamicGameState::outcome();
/// however, if this is reached we return WinDrawOutcome::Win(last player).
pub struct EndStateEvaluator<G>
where
    G: GameState,
    G::Action: 'static,
{
    visited: HashMap<G, G::Outcome>,
}

impl<G> Evaluator<G> for EndStateEvaluator<G>
where
    G: GameState<Outcome = WinDrawOutcome<G>> + Hash + Eq,
    G::Outcome: Eq + Clone,
    G::Player: Eq,
{
    type Evaluation = G::Outcome;

    fn evaluate(&mut self, state: &G, action: &G::Action) -> GameResult<Self::Evaluation, G> {
        // Get new state.
        let new_state = state.apply(action)?;
        // If state already visited and evaluated, return the outcome.
        if let Some(outcome) = self.visited.get(&new_state) {
            // Clone cause visited owns *outcome.
            return Ok(outcome.clone());
        } else if let Some(outcome) = new_state.outcome() {
            // Check if we're in a final state, if so cache it and return.
            self.visited.insert(new_state, outcome.clone());
            return Ok(outcome);
        }
        // Couldn't immediately tell what the value is, so recurse.
        let mut actions = new_state.legal_actions().peekable();
        if actions.peek().is_none() {
            // Drop cause actions borrows new_state.
            drop(actions);
            return Err(GameError::NoLegalActions(new_state));
        }
        let current_player = new_state.current_player();
        let mut n_draws = 0;
        let mut n_losses = 0;
        for action in actions {
            let outcome = self.evaluate(&new_state, action)?;
            match &outcome {
                WinDrawOutcome::Draw => n_draws += 1,
                WinDrawOutcome::Win(player) => {
                    // If a winning move for the current player exists, return it.
                    if *player == current_player {
                        return Ok(outcome);
                    } else {
                        n_losses += 1;
                    }
                }
            }
        }
        // All moves available to the current player result in a loss or a draw. If there are no
        // draws available, then the only options for the player are losses.
        if n_draws > 0 {
            // If there is one draw available to the current player then the evaluation of the
            // original action is a draw.
            Ok(WinDrawOutcome::Draw)
        } else if n_losses > 0 {
            // Recall state refers to the original state i.e. from the perspective of the caller.
            // If there are no draws for the current player
            // then the evaluation of this position is a win for the last player.
            // TODO: This would be an odd position to be in. Ideally, we would have identified
            // that this is a losing position for the current player up higher in the function,
            // where we call new_state.outcome() i.e. we should have:
            // new_state.outcome() == Some(WinDrawOutCome::Win(last_player))
            Ok(WinDrawOutcome::Win(state.current_player()))
        } else {
            // If this is reached then there were no legal actions available.
            Err(GameError::NoLegalActions(new_state))
        }
    }
}

impl<G> EndStateEvaluator<G>
where
    G: GameState<Outcome = WinDrawOutcome<G>> + Hash + Eq,
    G::Outcome: Eq,
{
    #[allow(dead_code, unused_variables)]
    fn evaluate_unchecked(&mut self, state: &G, action: &G::Action) -> G::Outcome {
        todo!()
    }

    pub fn new() -> Self {
        Self {
            visited: HashMap::new(),
        }
    }
}

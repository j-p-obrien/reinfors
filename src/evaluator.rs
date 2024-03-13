use std::{collections::HashMap, hash::Hash};

use crate::game_state::{
    outcome::WinDraw::{self, *},
    player::TwoPlayer,
    ApplyResult::*,
    GameState,
};

pub trait Evaluator<G>
where
    G: GameState,
{
    type Evaluation;

    fn evaluate(&mut self, state: &G, action: &G::Action) -> Self::Evaluation;
}

pub trait ToEvaluation<G>: Evaluator<G>
where
    G: GameState,
{
    fn to_evaluation(&self, player: &G::Player, outcome: &G::Outcome) -> Self::Evaluation;
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
    G: GameState,
{
    type Evaluation = u64;

    fn evaluate(&mut self, _state: &G, _action: &G::Action) -> Self::Evaluation {
        const A: u64 = 1664525;
        const C: u64 = 1013904223;

        self.rng_state = self.rng_state.wrapping_mul(A).wrapping_add(C);
        self.rng_state
    }
}

/// This evaluator recurses through the legal actions available at each stage of the game and thus
/// MAY BE VERY EXPENSIVE TO COMPUTE!!! This evaluator is completely infeasible to compute for
/// anything more than very simple games.
#[derive(Debug)]
pub struct MinimaxEvaluator<G> {
    visited: HashMap<G, i8>,
}

impl<G> MinimaxEvaluator<G>
where
    G: GameState<Outcome = WinDraw<G>, Player = TwoPlayer>,
{
    /// Computes the evaluation from the perspective of the given player.
    fn outcome_to_eval(&self, player: &G::Player, outcome: &G::Outcome) -> i8 {
        match outcome {
            Win(same_player) if player == same_player => 1,
            Draw => 0,
            Win(_) => -1,
        }
    }

    pub fn new() -> Self {
        Self {
            visited: HashMap::new(),
        }
    }
}

impl<G> Evaluator<G> for MinimaxEvaluator<G>
where
    G: GameState<Outcome = WinDraw<G>, Player = TwoPlayer> + Hash + Eq,
{
    type Evaluation = i8;

    /// If the given action results in a terminal state, returns the Evaluation of that state for
    /// the caller. Afterwards, checks if the new state has already been evaluated and returns that
    /// evaluation if available. Otherwise, we evaluate all of the legal actions available to the
    /// next player and return the Evaluaton (from the perpective of the caller) of the most
    /// favorable action for the opponent.
    fn evaluate(&mut self, state: &G, action: &G::Action) -> Self::Evaluation {
        // Keep track of who called evaluate.
        let original_player = state.current_player();
        // Get new state.
        let new_state = match state.apply(action) {
            Ongoing(state) => state,
            Finished(_, outcome) => return self.outcome_to_eval(&original_player, &outcome),
        };
        // If state already visited and evaluated, return the outcome.
        if let Some(&eval) = self.visited.get(&new_state) {
            return eval;
            // Check if we're in a final state, if so cache it and return.
        };
        // Couldn't immediately tell what the value is, so recurse.
        let mut eval = 1;
        let mut actions = new_state.legal_actions();
        while let Some(new_action) = actions.next() {
            // This outcome is from the perspective of the player of new_state, i.e. the opponent
            // of the caller.
            let opponent_outcome = self.evaluate(&new_state, new_action);
            // This means that the opponent has a winning move, thus, the evaluation here is -1
            // for the caller.
            if opponent_outcome == 1 {
                // drop is necessary because actions borrows new_state.
                drop(actions);
                self.visited.insert(new_state, -1);
                return -1;
            // This means that the opponent has a draw available to them, so we assume they will
            // take it if there are no winning moves for them.
            } else if opponent_outcome == 0 {
                eval = 0;
            }
        }
        // This is necessary because actions borrows new_state
        drop(actions);
        self.visited.insert(new_state, eval);
        eval
    }
}

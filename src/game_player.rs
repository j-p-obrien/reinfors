use crate::{evaluator::Evaluator, game_state::*, strategy::Strategy};
use std::fmt::{Debug, Display};

#[derive(Debug)]
pub struct GamePlayer<G, E, S>
where
    G: GameState,
    E: Evaluator<G>,
    S: Strategy<G, E>,
{
    pub state: G,
    pub evaluator: E,
    pub strategy: S,
}

impl<G, E, S> GamePlayer<G, E, S>
where
    G: GameState,
    E: Evaluator<G>,
    S: Strategy<G, E>,
{
}

impl<G, E, S> GamePlayer<G, E, S>
where
    G: GameState,
    E: Evaluator<G>,
    S: Strategy<G, E>,
{
    pub fn new(state: G, evaluator: E, strategy: S) -> Self {
        Self {
            state,
            evaluator,
            strategy,
        }
    }

    pub fn state(&self) -> &G {
        &self.state
    }

    pub fn evaluator(&mut self) -> &mut E {
        &mut self.evaluator
    }

    pub fn strategy(&mut self) -> &mut S {
        &mut self.strategy
    }

    pub fn into_constituents(self) -> (G, E, S) {
        (self.state, self.evaluator, self.strategy)
    }

    pub fn play(&mut self) -> (G, G::Outcome) {
        loop {
            let best_action = self.strategy.choose(&self.state, &mut self.evaluator);
            match self.state.apply(&best_action) {
                Ongoing(new_state) => {
                    self.state = new_state;
                }
                Finished(new_state, outcome) => return (new_state, outcome),
            }
        }
    }

    pub fn play_display(&mut self) -> (G, G::Outcome)
    where
        G: Display,
    {
        loop {
            print!("{}", self.state);
            let best_action = self.strategy.choose(&self.state, &mut self.evaluator);
            match self.state.apply(&best_action) {
                Ongoing(new_state) => {
                    self.state = new_state;
                }
                Finished(new_state, outcome) => {
                    print!("{}", new_state);
                    return (new_state, outcome);
                }
            }
        }
    }

    pub fn play_interactive(&mut self, player_starts: bool) -> (G, G::Outcome)
    where
        G: Display + Interactive,
    {
        print!("{}", self.state);
        if player_starts {
            let action = self.state.get_user_input();
            match self.state.apply(&action) {
                Ongoing(new_state) => {
                    self.state = new_state;
                }
                Finished(new_state, outcome) => return (new_state, outcome),
            }
            print!("{}", self.state);
        }
        loop {
            let best_action = self.strategy.choose(&self.state, &mut self.evaluator);
            match self.state.apply(&best_action) {
                Ongoing(new_state) => self.state = new_state,
                Finished(new_state, outcome) => {
                    print!("{}", new_state);
                    return (new_state, outcome);
                }
            }
            print!("{}", self.state);
            let action = self.state.get_user_input();
            match self.state.apply(&action) {
                Ongoing(new_state) => {
                    self.state = new_state;
                }
                Finished(new_state, outcome) => {
                    print!("{}", new_state);
                    return (new_state, outcome);
                }
            }
            print!("{}", self.state);
        }
    }
}

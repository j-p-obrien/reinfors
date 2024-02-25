use crate::{evaluator::*, game_state::*, strategy::*};
use std::fmt::{Debug, Display};

#[derive(Debug)]
pub struct GamePlayer<G, E, S>
where
    G: DynamicGameState,
    E: Evaluator<G>,
    S: Strategy<G, E>,
{
    state: G,
    evaluator: E,
    strategy: S,
}

impl<G, E, S> GamePlayer<G, E, S>
where
    G: DynamicGameState,
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

    pub fn play(&mut self) -> GameResult<G::Outcome, G> {
        loop {
            if let Some(outcome) = self.state.outcome() {
                return Ok(outcome);
            }
            let best_action = self
                .strategy
                .best_action(&self.state, &mut self.evaluator)?;
            self.state.apply_mut(&best_action)?
        }
    }

    pub fn play_display(&mut self) -> GameResult<G::Outcome, G>
    where
        G: Display,
    {
        loop {
            print!("{}", self.state);
            if let Some(outcome) = self.state.outcome() {
                return Ok(outcome);
            }
            let best_action = self
                .strategy
                .best_action(&self.state, &mut self.evaluator)?;
            self.state.apply_mut(&best_action)?
        }
    }

    pub fn play_interactive(&mut self, player_is: usize) -> GameResult<G::Outcome, G>
    where
        G: Display,
        G: Interactive,
        E: Debug,
    {
        if player_is == 1 {
            println!("Board positions are as follows:\n8|7|6\n5|4|3\n2|1|0");
            println!("Press a number between 0 and 8 inclusive and hit enter.");
            while let Some(action) = self.state.get_user_input() {
                match self.state.apply_mut(&action) {
                    Ok(_) => break,
                    Err(_) => println!("Try again."),
                }
            }
            print!("{}", self.state);
        }
        loop {
            println!("Computer going...");
            let best_action = self
                .strategy
                .best_action(&self.state, &mut self.evaluator)?;
            //dbg!(&self.evaluator);
            self.state.apply_mut(&best_action)?;
            print!("{}", self.state);
            if let Some(outcome) = self.state.outcome() {
                return Ok(outcome);
            }
            println!("Board positions are as follows:\n8|7|6\n5|4|3\n2|1|0");
            println!("Press a number between 0 and 8 inclusive and hit enter.");
            while let Some(action) = self.state.get_user_input() {
                match self.state.apply_mut(&action) {
                    Ok(_) => break,
                    Err(_) => println!("Try again."),
                }
            }
            print!("{}", self.state);
            if let Some(outcome) = self.state.outcome() {
                return Ok(outcome);
            }
        }
    }
}

pub mod error;
pub mod outcome;
pub mod player;

pub use ApplyResult::*;

use self::{outcome::WinDraw, player::TwoPlayer};

/// The result of applying an action to the Game.
pub enum ApplyResult<G>
where
    G: GameState,
{
    ///
    Ongoing(G),
    Finished(G, G::Outcome),
}

/// The main trait for Games.
pub trait GameState: Sized {
    /// The type of Actions associated with the Game.
    type Action;

    /// The type of Players associated with the Game. Games can be one, two, or multiplayer and
    /// thus we leave the flexibility for this.
    type Player;

    /// The type of Outcome associated with the Game. Some games are Win/Loss/Draw, some have
    /// numeric outcomes, and some may have some kind of exotic kind of Outcome.
    type Outcome;

    /// Returns the result of applying the Action to the Game. Returns either the next state, if
    /// the Game is not over, or the Outcome, if the given move would result in a terminal state.
    fn apply(&self, action: &Self::Action) -> ApplyResult<Self>;

    /// Returns an Iterator over the legal actions in the current state. Note that the lifetime
    /// of the actions is tied to the state; this ensures that legal actions do not outlive the
    /// state they are legal for! This of course does not prevent you from creating a new state
    /// and applying an action for a previous state to it. Try not to do this.
    fn legal_actions(&self) -> impl Iterator<Item = &Self::Action>;

    /// Returns the current player of the game. Useful for implementing strategies and evaluators.
    fn current_player(&self) -> Self::Player;
}

/// Trait for Games where all possible actions are known ahead of time. This is for Games like
/// chess, checkers, tic-tac-toe, Connect 4 etc. where there is a fixed set of actions known at
/// compile-time.
pub trait EnumerableActions: GameState {
    /// Returns the index of the given action in the slice returned by actions() i.e. we should
    /// always have:
    /// actions()[action_index(action)] == *action
    fn action_index(&self, action: &Self::Action) -> usize;
}

/// Allows the user to separate applying an action and checking its outcome.
///
/// This is useful if computing the outcome is expensive, and it would be faster to instead do a
/// lookup and see if the outcome has already been computed. It is also useful in scenarios where
/// you have an action history and want to compute the resulting state from the history without
/// unnecessarily checking the outcome.
///
/// However, With great power comes great responsibility! Applying an action to a game that is
/// finished will lead to weird results. It is up to the caller to ensure that this does not
/// happen.
// pub trait ApplyUnchecked {
//     type Action;
//     type Outcome;
//     type Player;

//     fn apply_unchecked(&self, action: &Self::Action) -> Self;

//     fn outcome(&self) -> Option<Self::Outcome>;

//     fn current_player(&self) -> Self::Player;
// }

// pub trait GenerableActions {
//     type Action;
//     fn legal_actions(&self) -> impl Iterator<Item = &Self::Action>;
// }

// impl<G, A> GameState for G
// where
//     G: ApplyUnchecked<Action = A> + GenerableActions<Action = A>,
// {
//     type Action = <Self as ApplyUnchecked>::Action;

//     type Player = <Self as ApplyUnchecked>::Player;

//     type Outcome = <Self as ApplyUnchecked>::Outcome;

//     fn apply(&self, action: &Self::Action) -> ApplyResult<Self> {
//         let new_state = self.apply_unchecked(action);
//         match new_state.outcome() {
//             Some(outcome) => ApplyResult::Finished(new_state, outcome),
//             None => ApplyResult::Ongoing(new_state),
//         }
//     }

//     fn legal_actions(&self) -> impl Iterator<Item = &<Self as GameState>::Action> {
//         self.legal_actions()
//     }

//     fn current_player(&self) -> Self::Player {
//         self.current_player()
//     }
// }

pub trait TwoPlayerZeroSum: GameState {}

impl<G> TwoPlayerZeroSum for G where G: GameState<Player = TwoPlayer, Outcome = WinDraw<Self>> {}

pub trait PartialInformation: GameState {
    type PlayerView;

    fn view_as(&self, player: &Self::Player) -> Self::PlayerView;
}

pub trait Interactive: GameState {
    fn get_user_input(&self) -> Self::Action;
}

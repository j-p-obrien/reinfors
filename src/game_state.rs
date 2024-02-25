use std::fmt::Display;

/// Potential errors in the playing of the game.
#[derive(Debug)]
pub enum GameError<G>
where
    G: DynamicGameState,
{
    /// Should be returned if the user tries to apply an Action that is illegal in the current
    /// DynamicGameState.
    /// Returns the associated DynamicGameState and Action.
    IllegalAction(G, G::Action),
    /// Should be returned if the user tries to apply an action to a Game that is already over.
    /// Returns the associated DynamicGameState and Action
    GameOver(G, G::Action),
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

pub type GameResult<T, G> = Result<T, GameError<G>>;

pub trait UnsafeGameState: Sized {
    type Action;

    type Player;

    type Outcome;

    /// This function assumes that both the given action is legal and that the game is not in a
    /// terminal state. This allows performance gains when needed, but this makes the function
    /// unsafe as a result, as it is up to the caller to check invariants before proceeding. It is
    /// important to keep in mind that this function may return an "invalid" UnsafeGameState in the
    /// sense that it would not be valid to apply an Action to a game that has finished, or to
    /// apply an illegal action.
    unsafe fn unsafe_apply(&self, action: &Self::Action) -> Self;

    /// Returns true if the given Action is legal in the given State; false otherwise.
    fn is_legal(&self, action: &Self::Action) -> bool;

    /// Returns Some(Outcome) if the game is in a terminal state; otherwise None. If this function
    /// returns Some, then the game is over and you should not try to apply any actions to it.
    fn outcome(&self) -> Option<Self::Outcome>;

    /// Returns the current player of the game. Useful for implementing strategies and evaluators.
    fn current_player(&self) -> Self::Player;

    /// The same as unsafe_apply, but applies the action mutably. Again, this function assumes
    /// the given action is legal and the Game is not in a terminal state. It is important to keep
    /// in mind that after applying the Action, the underlying State may be "invalid" in some
    /// sense, and it is up to the caller to keep any invariants in mind.
    unsafe fn unsafe_apply_mut(&mut self, action: &Self::Action) {
        unsafe { *self = self.unsafe_apply(action) }
    }
}

pub enum ApplyResult<G>
where
    G: DynamicGameState,
{
    ///
    State(G),
    Outcome(G::Outcome),
}

/// Trait for Games where the available actions may not be known at compile-time or may be too
/// numerous to list e.g. in poker an action might be Bet(Amount) where Amount is a usize. In this
/// case, actions are generated on the fly as opposed to picking from a pre-determined list. If
/// your game is more like the latter, you should implement GameState as well.
pub trait DynamicGameState: Sized {
    /// The actions associated with the Game.
    type Action;

    /// The Player(s) associated with the Game. Since games can be one, two, or multiplayer,
    /// we leave flexibility for this.
    type Player;

    /// Different games can have different outcomes. For instance, in chess the outcome is
    /// win/loss/draw. In poker you might win a certain amount of money. You may also win/lose a
    /// certain amount, like in matrix games.
    type Outcome;

    /// Returns Ok(GameState) if move is legal, where the GameState is the result of applying the
    /// given action; otherwise, returns Err(GameError::IllegalAction).
    /// TODO: Consider whether or not we should return Self | Outcome | Err in this function.
    /// This could help avoid errors where the user tries to apply an action to a Game that has
    /// already ended.
    fn apply(&self, action: &Self::Action) -> GameResult<Self, Self>;

    /// If game is over, returns Some(Outcome); otherwise None.
    fn outcome(&self) -> Option<Self::Outcome>;

    /// Returns the current player.
    fn current_player(&self) -> Self::Player;

    /// Returns true if the given move is legal. You probably want to re-implement this for
    /// performance reasons but it is included for the lazy.
    fn is_legal(&self, action: &Self::Action) -> bool {
        match self.apply(action) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// Applies the given action to the game State. Returns OK(()) if the move was legal,
    /// Err(GameError::IllegalAction) otherwise.
    fn apply_mut(&mut self, action: &Self::Action) -> GameResult<(), Self> {
        let next_state = self.apply(action)?;
        *self = next_state;
        Ok(())
    }

    /// Returns true if the game has Some(Outcome); false otherwise
    fn is_finished(&self) -> bool {
        self.outcome().is_some()
    }
}

/// Trait for Games where all possible actions are known ahead of time. This is for Games like
/// chess, checkers, tic-tac-toe, Connect 4 etc. where there is a fixed set of actions known at
/// compile-time. Thus, the action should be static.
pub trait GameState: DynamicGameState
where
    Self::Action: 'static,
{
    /// Returns a slice of all the actions in the Game, regardless of legality.
    fn actions(&self) -> &'static [Self::Action];

    /// Returns an iterator over the legal actions in the current GameState.
    fn legal_actions(&self) -> impl Iterator<Item = &'static Self::Action> {
        self.actions()
            .into_iter()
            .filter(|&action| self.is_legal(action))
    }
}

pub trait Interactive: DynamicGameState {
    fn get_user_input(&self) -> Option<Self::Action>;
}

/// The Player type. Represents the players of a game. Supports both 0 and 1 based indexing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Player {
    n_players: usize,
    current: usize,
}

impl Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Player {}", self.current_player())
    }
}

impl Player {
    pub fn new(n_players: usize) -> Self {
        Self {
            n_players,
            current: 0,
        }
    }

    #[inline]
    fn wrapping_sub(&self) -> usize {
        if self.current == 0 {
            self.n_players - 1
        } else {
            self.current - 1
        }
    }

    // Returns the 0-indexed player number
    pub fn player_index(&self) -> usize {
        self.current
    }

    // Returns the 1-indexed player number
    pub fn current_player(&self) -> usize {
        self.current + 1
    }

    pub fn next(&self) -> Self {
        Self {
            n_players: self.n_players,
            current: (self.current + 1) % self.n_players,
        }
    }

    pub fn next_mut(&mut self) {
        self.current = (self.current + 1) % self.n_players
    }

    pub fn last(&self) -> Self {
        Self {
            n_players: self.n_players,
            current: self.wrapping_sub(),
        }
    }

    pub fn last_mut(&mut self) {
        self.current = self.wrapping_sub()
    }
}

/// The type of a Game outcome where there either is one definite winner or a draw. This is for
/// Games like chess, checkers, tic-tac-toe, Monopoly etc.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum WinDrawOutcome<G>
where
    G: DynamicGameState,
{
    Win(G::Player),
    Draw,
}

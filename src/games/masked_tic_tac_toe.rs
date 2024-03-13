use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    io::{self, BufRead},
    vec,
};

use crate::{
    evaluator::Evaluator,
    game_state::{
        outcome::{
            self,
            WinDraw::{self, *},
        },
        player::{self, TwoPlayer},
        ApplyResult, GameState, Interactive,
    },
};

use super::tic_tac_toe::*;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct BitBoard(u16);

impl BitBoard {
    #[inline]
    pub fn is_occupied(&self, action: &Action) -> bool {
        (self.0 & action.0) == action.0
    }

    #[inline]
    pub fn apply(&mut self, action: &Action) {
        self.0 |= action.0
    }
}

impl Debug for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut squares = vec![];
        for i in 0..9 {
            if (self.0 >> i) & 1 == 1 {
                squares.push(i)
            }
        }
        f.debug_list().entries(squares.iter()).finish()
        //f.debug_tuple("BitBoard").field(&squares).finish()
        //f.debug_tuple("BitBoard").field(&self.0).finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Info<T> {
    Visible(T),
    Masked(T),
    Invisible,
}

impl<T> Display for Info<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Info::Visible(piece) => write!(f, "{}", *piece),
            _ => write!(f, "▮"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaskedTicTacToe<const N: usize> {
    board: [BitBoard; 2],
    masked: [Action; N],
    no_action: BitBoard,
    history: Vec<Action>,
    current_player: TwoPlayer,
    player1_piece: Piece,
}

#[derive(Debug, Clone, Copy)]
enum Visibility {
    Vis(Piece),
    Num(u8),
    Invis,
}

impl Display for Visibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Visibility::Vis(piece) => write!(f, "{piece}"),
            Visibility::Num(i) => write!(f, "{i}"),
            Visibility::Invis => write!(f, "▮"),
        }
    }
}

impl<const N: usize> Display for MaskedTicTacToe<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let board: Vec<_> = ALL_ACTIONS
            .iter()
            .enumerate()
            .map(|(i, action)| {
                if self.is_masked(action) {
                    Visibility::Invis
                } else {
                    match self.player_piece(action) {
                        Piece::Empty => Visibility::Num(i as u8),
                        piece => Visibility::Vis(piece),
                    }
                }
            })
            .collect();
        write!(
            f,
            "{}|{}|{}\n{}|{}|{}\n{}|{}|{}\n\n",
            board[8],
            board[7],
            board[6],
            board[5],
            board[4],
            board[3],
            board[2],
            board[1],
            board[0],
        )
    }
}

impl<const N: usize> Default for MaskedTicTacToe<N> {
    fn default() -> Self {
        Self {
            board: Default::default(),
            masked: [Action(0); N],
            no_action: Default::default(),
            history: Default::default(),
            current_player: Default::default(),
            player1_piece: Default::default(),
        }
    }
}

impl<const N: usize> MaskedTicTacToe<N> {
    pub fn new(masked: [Action; N]) -> Self {
        Self {
            masked,
            ..Default::default()
        }
    }

    pub fn genesis(&self) -> Self {
        Self::new(self.masked)
    }

    pub fn history(&self) -> Vec<Action> {
        self.history.clone()
    }

    #[inline]
    pub fn last_player(&self) -> TwoPlayer {
        self.current_player.last()
    }

    pub fn masked_actions(&self) -> [Action; N] {
        self.masked
    }

    pub fn legal_masked(&self) -> impl Iterator<Item = &Action> {
        self.masked.iter().filter(|&action| self.is_legal(action))
    }

    #[inline]
    pub fn is_masked(&self, action: &Action) -> bool {
        self.masked
            .iter()
            .any(|masked_action| masked_action == action)
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        (self.board[0].0 | self.board[1].0) == FULL
    }

    pub fn last_player_wins(&self) -> bool {
        let board = self.board[self.last_player().index()];
        WINNING_POSITIONS
            .iter()
            .any(|&position| (position & board.0) == position)
    }

    /// Here, we assume that we are given a legal action.
    pub fn apply_unchecked(&self, action: &Action) -> Self {
        let mut board = self.board;
        let mut history = self.history.clone();
        let mut no_action = self.no_action;
        let last_player = self.last_player();
        // Note that the given action is legal i.e. even if the other player occupies the state we
        // can still attempt the move, it just will silently fail.
        if self.player_occupies(&last_player, action) {
            no_action.apply(action)
        } else {
            board[self.current_player.index()].apply(action);
        }
        history.push(*action);
        Self {
            board,
            history,
            no_action: no_action,
            masked: self.masked,
            current_player: last_player,
            player1_piece: self.player1_piece,
        }
    }

    pub fn apply_unchecked_mut(&mut self, action: &Action) {
        // Note that the given action is legal i.e. even if the other player occupies the state we
        // can still attempt the move, it just will silently fail.
        if self.player_occupies(&self.last_player(), action) {
            self.no_action.apply(action)
        } else {
            self.board[self.current_player.index()].apply(action);
        }
        self.history.push(*action);
        self.current_player.next_mut();
    }

    pub fn outcome(&self) -> Option<WinDraw<Self>> {
        if self.last_player_wins() {
            Some(Win(self.last_player()))
        } else if self.is_full() {
            Some(Draw)
        } else {
            None
        }
    }

    #[inline]
    fn player_occupies(&self, player: &TwoPlayer, action: &Action) -> bool {
        self.board[player.index()].is_occupied(action)
    }

    #[inline]
    fn player_index_occupies(&self, player_index: usize, action: &Action) -> bool {
        self.board[player_index].is_occupied(action)
    }

    pub fn is_legal(&self, action: &Action) -> bool {
        // If the current player occupies the spot, action is always illegal.
        if self.player_occupies(&self.current_player, action) {
            false
        // If this action is masked, it's legal only if we haven't moved here yet.
        } else if self.is_masked(action) {
            !self.no_action.is_occupied(action)
        // If we don't occupy, and the action is visible, then we can move there only if the other
        // player doesn't already occupy it.
        } else {
            !self.player_occupies(&self.last_player(), action)
        }
    }

    pub fn legal_actions(&self) -> impl Iterator<Item = &Action> {
        ALL_ACTIONS.iter().filter(|&action| self.is_legal(action))
    }

    pub fn visible_history(&self) -> Vec<Info<Action>> {
        // When looking at the history of the game, the first masked action is unique because it is
        // guaranteed to succeed. Thus, this action is visible to the player making the move. No
        // other masked action has this property.
        let mut first_masked_action = true;
        self.history
            .iter()
            .enumerate()
            .map(|(i, action)| {
                if self.is_masked(action) {
                    // If the action is masked and it was the other player that made it, it is
                    // always invisible to us.
                    let info = if i % 2 != self.current_player().index() {
                        Info::Invisible
                    // If twe made the move and it was the first masked action, then this action
                    // is visible to us
                    } else if first_masked_action {
                        Info::Visible(*action)
                    // Otherwise, we know we moved here, but the result of the move is hidden
                    // (masked) from us.
                    } else {
                        Info::Masked(*action)
                    };
                    first_masked_action = false;
                    info
                } else {
                    Info::Visible(*action)
                }
            })
            .collect()
    }

    pub fn player_piece(&self, action: &Action) -> Piece {
        if self.player_index_occupies(0, action) {
            self.player1_piece
        } else if self.player_index_occupies(1, action) {
            self.player1_piece.flip()
        } else {
            Piece::Empty
        }
    }
}

impl<const N: usize> Interactive for MaskedTicTacToe<N> {
    fn get_user_input(&self) -> Self::Action {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            if let Ok(line) = line {
                if let Ok(num) = line.parse::<u16>() {
                    if num <= 8 {
                        return Action(1 << num);
                    } else {
                        println!("Try again");
                    }
                } else {
                    println!("Try again");
                }
            } else {
                println!("Try again");
            }
        }
        unreachable!()
    }
}

impl<const N: usize> GameState for MaskedTicTacToe<N> {
    type Action = Action;

    type Player = TwoPlayer;

    type Outcome = WinDraw<Self>;

    fn apply(&self, action: &Self::Action) -> ApplyResult<Self> {
        let next_state = self.apply_unchecked(action);
        if let Some(outcome) = next_state.outcome() {
            ApplyResult::Finished(next_state, outcome)
        } else {
            ApplyResult::Ongoing(next_state)
        }
    }

    fn legal_actions(&self) -> impl Iterator<Item = &Self::Action> {
        self.legal_actions()
    }

    fn current_player(&self) -> Self::Player {
        self.current_player
    }
}

#[derive(Debug, Clone)]
pub struct MaskedEvaluator {
    pub visited: HashMap<(Vec<Info<Action>>, Action), (i8, i8)>,
}

impl MaskedEvaluator {
    pub fn new() -> Self {
        Self {
            visited: HashMap::new(),
        }
    }

    pub fn evaluate<const N: usize>(
        &mut self,
        state: &MaskedTicTacToe<N>,
        action: &Action,
    ) -> (i8, i8) {
        // If we were to use apply to compute the outcome of the action
        // then we would be cheating! It is not clear whether we know exactly what state we
        // are in at the moment because some moves are masked. Thus, if an outcome were to be
        // returned, we would be seeing the future. Instead, we check to see if this sequence of
        // actions has already been visited.
        let history = state.visible_history();
        //dbg!(history.len());
        if history.len() > 11 {
            dbg!(&history);
            unreachable!("How did you manage to go more than 11 times")
        }
        // We have to do this cause otherwise the if let takes ownership of history.
        let history_tuple = (history, *action);
        if let Some(&eval) = self.visited.get(&history_tuple) {
            return eval;
        }
        // move back out the tuple
        let history = history_tuple.0;
        // This computes all of the potential current states we could be in given the history of
        // actions. Now, since we know exactly what state(s) we are in, it is ok to peek at the
        // result from applying an action.
        let superposition = self.superposition(state.genesis(), &history);
        let current_player = state.current_player;
        let (mut my_eval, mut their_eval) = (1, 1);
        for possible_current_state in superposition {
            // Compute one of the potential reachable states.
            if !possible_current_state.is_legal(action) {
                continue;
                // p sure this branch should be unreachable, the hidden moves shouldn't effect legality
                //unreachable!()
            }
            let possible_next_state = possible_current_state.apply_unchecked(action);
            // Check outcome.
            match possible_next_state.outcome() {
                Some(outcome) => match outcome {
                    // If the outcome is a Win, it's a win for the current player. The other player
                    // cannot win after one of our moves. Note however that this doesn't imply that
                    // applying this move means we win! This state is only a potential one, we don't
                    // actually know whether or not we are in this state.
                    Win(player) if player == current_player => their_eval = -1,
                    // If the outcome is a draw, then we can only guarantee at most a draw
                    Draw => {
                        (my_eval, their_eval) = (my_eval.min(0), their_eval.min(0));
                    }
                    Win(_) => unreachable!("Other player shouldn't win after one of our moves."),
                },
                None => {
                    let opponent_actions = possible_next_state.legal_actions();
                    let mut their_step_ahead_eval = -1;
                    for opponent_action in opponent_actions {
                        // This evaluation is from the opponent's perspective
                        let (their_temp_eval, my_temp_eval) =
                            self.evaluate(&possible_next_state, opponent_action);
                        (my_eval, their_step_ahead_eval) = match (their_temp_eval, my_temp_eval) {
                            (1, 1) => {
                                unreachable!()
                            }
                            (1, 0) => {
                                unreachable!()
                            }
                            // If they have a move that wins in all superpositions, our eval is -1
                            // And theirs is 1.
                            (1, -1) => (-1, 1),
                            (0, 1) => {
                                unreachable!()
                            }
                            (0, 0) => (my_eval.min(0), their_step_ahead_eval.max(0)),
                            (0, -1) => (-1, their_step_ahead_eval.max(0)),
                            (-1, 1) => (my_eval, their_step_ahead_eval),
                            (-1, 0) => (my_eval.min(0), their_step_ahead_eval),
                            (-1, -1) => (-1, their_step_ahead_eval),
                            _ => unreachable!(),
                        }
                    }
                    their_eval = their_eval.min(their_step_ahead_eval)
                }
            }
        }
        self.visited
            .insert((history, *action), (my_eval, their_eval));
        (my_eval, their_eval)
    }

    fn superposition<const N: usize>(
        &self,
        genesis: MaskedTicTacToe<N>,
        history: &[Info<Action>],
    ) -> Vec<MaskedTicTacToe<N>> {
        let mut superposition = vec![genesis];
        for observed in history {
            match &observed {
                // Apply known actions to each state we have. Note that if a given action results in
                // game over, we can safely conclude that we are not in that branch of the game
                // tree, as we would already know the outcome.
                Info::Visible(action) | Info::Masked(action) => {
                    superposition = superposition
                        .into_iter()
                        .filter(|state| state.is_legal(action))
                        .map(|state| state.apply_unchecked(action))
                        .filter(|new_state| new_state.outcome().is_none())
                        .collect()
                }
                Info::Invisible => {
                    let mut temp = vec![];
                    for state in &superposition {
                        for action in state.legal_masked() {
                            let new_state = state.apply_unchecked(action);
                            if new_state.outcome().is_none() {
                                temp.push(new_state)
                            }
                        }
                    }
                    superposition = temp;
                }
            }
        }
        superposition
    }
}

impl<const N: usize> Evaluator<MaskedTicTacToe<N>> for MaskedEvaluator {
    type Evaluation = (i8, i8);

    fn evaluate(&mut self, state: &MaskedTicTacToe<N>, action: &Action) -> Self::Evaluation {
        self.evaluate(state, action)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        evaluator::Evaluator,
        game_state::{outcome::WinDraw, player::TwoPlayer},
        games::{
            masked_tic_tac_toe::{MaskedEvaluator, MaskedTicTacToe},
            tic_tac_toe::{Action, ALL_ACTIONS},
        },
    };

    static MASKED: [Action; 2] = [ALL_ACTIONS[0], ALL_ACTIONS[1]];

    #[test]
    fn test_apply() {
        let mut game = MaskedTicTacToe::new(MASKED);
        game.apply_unchecked_mut(&MASKED[0]);
        game.apply_unchecked_mut(&MASKED[0]);
        game.apply_unchecked_mut(&MASKED[1]);
        game.apply_unchecked_mut(&MASKED[1]);
        game.apply_unchecked_mut(&ALL_ACTIONS[2]);
        assert_eq!(game.outcome(), Some(WinDraw::Win(TwoPlayer::default())))
    }

    #[test]
    fn test_legality_and_p0_win() {
        let mut game = MaskedTicTacToe::new(MASKED);

        // P0 View
        assert!(game.is_legal(&ALL_ACTIONS[0]));
        game.apply_unchecked_mut(&ALL_ACTIONS[0]);

        // P1 View
        dbg!(game.visible_history());
        assert!(game.is_legal(&ALL_ACTIONS[0]));
        assert!(game.is_legal(&ALL_ACTIONS[1]));
        game.apply_unchecked_mut(&ALL_ACTIONS[1]);

        // P0 view
        dbg!(game.visible_history());
        assert!(!game.is_legal(&ALL_ACTIONS[0]));
        assert!(game.is_legal(&ALL_ACTIONS[1]));
        game.apply_unchecked_mut(&ALL_ACTIONS[1]);

        // P1 View
        dbg!(game.visible_history());
        assert!(game.is_legal(&ALL_ACTIONS[0]));
        assert!(!game.is_legal(&ALL_ACTIONS[1]));
        game.apply_unchecked_mut(&ALL_ACTIONS[0]);

        // P0 View
        // let mut evaluator = MaskedEvaluator::new();
        // assert!(evaluator.evaluate(&game, &ALL_ACTIONS[4]) == 1);
        dbg!(game.visible_history());
        assert!(!game.is_legal(&ALL_ACTIONS[1]));
        assert!(!game.is_legal(&ALL_ACTIONS[0]));
        game.apply_unchecked_mut(&ALL_ACTIONS[4]);

        // P1 View
        dbg!(game.visible_history());
        assert!(!game.is_legal(&ALL_ACTIONS[1]));
        assert!(!game.is_legal(&ALL_ACTIONS[0]));
        assert!(!game.is_legal(&ALL_ACTIONS[4]));
        game.apply_unchecked_mut(&ALL_ACTIONS[6]);

        // P0 View
        dbg!(game.visible_history());
        assert!(game.is_legal(&ALL_ACTIONS[8]));
        game.apply_unchecked_mut(&ALL_ACTIONS[8]);
        assert_eq!(game.outcome(), Some(WinDraw::Win(TwoPlayer::new(true))));

        dbg!(&game);
    }
}

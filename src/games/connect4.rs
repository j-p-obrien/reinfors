use std::ops::Index;

/// An implementation of Connect 4.
///
/// This game is solved, and we know that Player 1 has a winning strategy. Using a minimax
/// evaluator and greedy strategy should always guarantee a win for Player 1. Sorry Player 2!
use crate::game_state::outcome::WinDraw;
use crate::game_state::player::TwoPlayer;
use crate::game_state::ApplyResult;
use crate::game_state::GameState;

const BOARD_WIDTH: usize = 7;
const BOARD_HEIGHT: usize = 6;
const FULL_ROW: u8 = 0b1111111;
const FIRST_FOUR: u8 = 0b1111;

type Column = u8;
type RowIdx = u8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BitBoard([u8; BOARD_HEIGHT]);

pub struct BoardRow(u8);

impl Index<RowIdx> for BitBoard {
    type Output = BoardRow;

    fn index(&self, index: RowIdx) -> &Self::Output {
        todo!()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct Connect4 {
    board: [BitBoard; 2],
    current_player: TwoPlayer,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Action(Column);

pub static ALL_MOVES: [Action; BOARD_WIDTH] = [
    Action(0),
    Action(1),
    Action(2),
    Action(3),
    Action(4),
    Action(5),
    Action(6),
];

impl Connect4 {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn current_player(&self) -> TwoPlayer {
        self.current_player
    }

    fn current_player_index(&self) -> usize {
        self.current_player.index()
    }

    pub fn legal_actions(&self) -> impl Iterator<Item = &Action> {
        ALL_MOVES.iter().filter(|&action| self.is_legal(action))
    }

    pub fn is_legal(&self, action: &Action) -> bool {
        self.col_is_empty(BOARD_HEIGHT, action)
    }

    fn rows(&self) -> impl Iterator<Item = Row> {
        (0..BOARD_HEIGHT).into_iter()
    }

    /// Bitwise OR the two rows together, shift right by the column number, and check if the
    /// rightmost bit is 0.
    fn col_is_empty(&self, row: Row, action: &Action) -> bool {
        (((self.board[0][row] | self.board[1][row]) >> action.0) & 1) == 0
    }

    /// Returns the index of the first row with the given column empty
    fn first_empty_row(&self, action: &Action) -> Option<usize> {
        self.rows().find(|&row| self.col_is_empty(row, action))
    }

    /// Computes the outcome of the game, if there is one. For Connect4, We only need to check if
    /// the last player won.
    fn outcome(&self, row: Row, action: &Action) -> Option<WinDraw<Self>> {
        let last_player = self.current_player.last();
        let board = self.board[last_player.index()];
        if self.row_winner(board[row])
            | self.col_winner(row, action, board)
            | self.diag_winner(row, action, board)
        {
            Some(WinDraw::Win(last_player))
        } else if self.top_row_full() {
            Some(WinDraw::Draw)
        } else {
            None
        }
    }

    fn row_winner(&self, board_row: u8) -> bool {
        (0..4).any(|shift| ((board_row >> shift) & FIRST_FOUR) == FIRST_FOUR)
    }

    fn col_winner(&self, row: Row, action: &Action, board: [u8; BOARD_HEIGHT]) -> bool {
        if row < 3 {
            false
        } else {
            (1..4).all(|i| ((board[row - i] >> action.0) & 1) == 1)
        }
    }

    fn diag_winner(&self, row: usize, action: &Action, board: [u8; BOARD_HEIGHT]) -> bool {
        todo!()
    }

    fn top_row_full(&self) -> bool {
        (self.board[0][BOARD_HEIGHT] | self.board[1][BOARD_HEIGHT]) == FULL_ROW
    }

    fn apply_action(&self, action: &Action) -> (Self, Row) {
        let mut new_board = self.board;
        let row = self
            .first_empty_row(action)
            .expect("Expected column to be empty.");
        new_board[self.current_player_index()][row] |= action.0;
        let next_player = self.current_player.next();
        (
            Self {
                board: new_board,
                current_player: next_player,
            },
            row,
        )
    }
}

impl GameState for Connect4 {
    type Action = Action;

    type Player = TwoPlayer;

    type Outcome = WinDraw<Self>;

    fn apply(&self, action: &Self::Action) -> ApplyResult<Self> {
        let (new_state, row) = self.apply_action(action);
        if let Some(outcome) = new_state.outcome(row, action) {
            ApplyResult::Finished(new_state, outcome)
        } else {
            ApplyResult::Ongoing(new_state)
        }
    }

    fn legal_actions(&self) -> impl Iterator<Item = &Self::Action> {
        self.legal_actions()
    }

    fn current_player(&self) -> Self::Player {
        self.current_player
    }
}

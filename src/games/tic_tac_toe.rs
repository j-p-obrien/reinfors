use crate::game_state::{
    DynamicGameState, GameError, GameResult, GameState, Interactive, Player, WinDrawOutcome,
};
use std::{
    fmt::Display,
    io::{self, BufRead},
};

/// We will encode positions using a bitboard. Square 0 is the lower right position on the board
/// and we count from right to left then bottom to top, like so:
/// 8 | 7 | 6
/// 5 | 4 | 3
/// 2 | 1 | 0
/// The rightmost logical bit of the u16 Board is Square 0. Bits are 1 if occupied and 0
/// otherwise. It would make sense to make this a struct to ensure that the positions are always
/// valid, but since this is not exposed to the user I will not add the extra boilerplate
/// necessary for this. Board represents the whole board, Square is just a single spot on the
/// Board.
type Board = u16;
type Position = u16;

/// Represents a move. A single 1 bit denotes which position to move to. Note that only the 9
/// rightmost logical bits may be 1, since we have only 9 squares.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Action(Position);

/// Used to represent the pieces on the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum Piece {
    #[default]
    X,
    O,
    Empty,
}

/// The state of the board. player1 and player2 encode the position for Player 1 and Player 2,
/// respectively. to_move encodes which player's turn it is. player1_piece encodes whether player
/// 1 is X's or O's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BoardState {
    board: [Board; 2],
    current_player: Player,
    player1_piece: Piece,
}

/// This encodes the winning positions. If A is the position of a player, then the player is in
/// a winning position only if (A & WINNING_POSITIONS[i]) == WINNING_POSITIONS[i] for some i.
static WINNING_POSITIONS: [Board; 8] = [
    // Top row
    0b111_000_000,
    // Middle row
    0b000_111_000,
    // Bottom row
    0b000_000_111,
    // Left column
    0b100_100_100,
    // Middle column
    0b010_010_010,
    // Right column
    0b001_001_001,
    // Upper Left Diagonal
    0b100_010_001,
    // Upper Right Diagonal
    0b001_010_100,
];

/// Array of all potential moves that can be made. These are all u16's with a single 1 bit i.e.
/// Move(2^k) represents moving to position k on the board.
pub static ALL_MOVES: [Action; 9] = [
    Action(1),
    Action(2),
    Action(4),
    Action(8),
    Action(16),
    Action(32),
    Action(64),
    Action(128),
    Action(256),
];

/// If all of these positions are occupied and there is no winner yet then the game is a draw.
/// If A and B are the positions of players A and B then the game is a draw if and only if:
/// (A + B) == DRAW
const DRAW: Board = 0b111_111_111;

impl Piece {
    pub fn flip(&self) -> Piece {
        match *self {
            Piece::X => Piece::O,
            Piece::O => Piece::X,
            Piece::Empty => Piece::Empty,
        }
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let piece = match *self {
            Piece::X => "X",
            Piece::O => "O",
            Piece::Empty => "_",
        };
        write!(f, "{}", piece)
    }
}

impl Display for BoardState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let board = self.pieces();
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

impl Default for BoardState {
    fn default() -> Self {
        Self {
            board: [0, 0],
            current_player: Player::new(2),
            player1_piece: Default::default(),
        }
    }
}

impl BoardState {
    /// Starts a new Game with the given piece for Player 1.
    pub fn new(player1_piece: Piece) -> Self {
        Self {
            player1_piece,
            ..Default::default()
        }
    }

    /// Returns true if the given move is legal i.e. the desired position is unoccupied
    pub fn is_legal(&self, action: &Action) -> bool {
        let filled_positions = self.board[0] | self.board[1];
        (action.0 & filled_positions) == 0
    }

    /// Applies the given action and returns the resulting BoardState. Does not check legality of
    /// the given action.
    pub fn apply_unchecked(&self, action: &Action) -> Self {
        let current_player = self.current_player.player_index();
        let mut board = self.board.clone();
        board[current_player] |= action.0;
        Self {
            board,
            current_player: self.current_player.next(),
            player1_piece: self.player1_piece,
        }
    }

    /// Mutably applies the given action. Does not check legality of the given action.
    pub fn apply_mut_unchecked(&mut self, action: &Action) {
        let current_player = self.current_player.player_index();
        self.board[current_player] |= action.0;
        self.current_player.next_mut();
    }

    pub fn outcome(&self) -> Option<WinDrawOutcome<Self>> {
        /*if self.is_draw() {
            Some(WinDrawOutcome::Draw)
        } else {
            self.winner()
        }*/

        if let Some(winner) = self.winner() {
            Some(winner)
        } else if self.is_draw() {
            Some(WinDrawOutcome::Draw)
        } else {
            None
        }
    }

    /// If there is a winner, returns Some(Win(Player)); otherwise None.
    pub fn winner(&self) -> Option<WinDrawOutcome<Self>> {
        let last_player = self.current_player.last();
        if WINNING_POSITIONS
            .iter()
            .any(|&pos| pos & self.board[last_player.player_index()] == pos)
        {
            Some(WinDrawOutcome::Win(last_player))
        } else if WINNING_POSITIONS
            .iter()
            .any(|&pos| pos & self.board[self.current_player.player_index()] == pos)
        {
            Some(WinDrawOutcome::Win(self.current_player))
        } else {
            None
        }
    }

    /// Returns true if the game is a draw; false otherwise.
    pub fn is_draw(&self) -> bool {
        self.board[0] + self.board[1] == DRAW
    }

    fn pieces(&self) -> [Piece; 9] {
        let mut buffer = [Piece::Empty; 9];
        buffer.iter_mut().enumerate().for_each(|(i, piece)| {
            if self.player_occupies(0, i) {
                *piece = self.player1_piece
            } else if self.player_occupies(1, i) {
                *piece = self.player1_piece.flip()
            }
        });
        buffer
    }

    #[inline]
    fn player_occupies(&self, player: usize, i: usize) -> bool {
        (self.board[player as usize] >> i) & 1 == 1
    }
}

impl DynamicGameState for BoardState {
    type Action = Action;

    type Player = Player;

    type Outcome = WinDrawOutcome<Self>;

    fn apply(&self, action: &Self::Action) -> GameResult<Self, Self> {
        if self.is_legal(action) {
            Ok(self.apply_unchecked(action))
        } else {
            Err(GameError::IllegalAction(self.clone(), action.clone()))
        }
    }

    fn outcome(&self) -> Option<Self::Outcome> {
        self.outcome()
    }

    fn current_player(&self) -> Self::Player {
        self.current_player
    }

    fn is_legal(&self, action: &Self::Action) -> bool {
        self.is_legal(action)
    }

    fn apply_mut(&mut self, action: &Self::Action) -> GameResult<(), Self> {
        if self.is_legal(action) {
            self.apply_mut_unchecked(action);
            Ok(())
        } else {
            Err(GameError::IllegalAction(self.clone(), action.clone()))
        }
    }

    fn is_finished(&self) -> bool {
        self.outcome().is_some()
    }
}

impl GameState for BoardState {
    fn actions(&self) -> &'static [Self::Action] {
        &ALL_MOVES
    }
}

impl Interactive for BoardState {
    fn get_user_input(&self) -> Option<Self::Action> {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            if let Ok(line) = line {
                if let Ok(num) = line.parse::<u16>() {
                    if num <= 8 {
                        return Some(Action(1 << num));
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
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::games::tic_tac_toe::*;

    #[test]
    fn test1() {
        let mut board1 = BoardState::default();
        board1.apply_mut_unchecked(&Action(1));
        board1.apply_mut_unchecked(&Action(8));
        board1.apply_mut_unchecked(&Action(2));
        board1.apply_mut_unchecked(&Action(16));
        board1.apply_mut_unchecked(&Action(4));

        assert_eq!(board1.outcome(), Some(WinDrawOutcome::Win(Player::new(2))))
    }
}

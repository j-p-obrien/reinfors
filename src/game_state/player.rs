use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct OnePlayer;

impl Display for OnePlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Player 0")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TwoPlayer {
    player0: bool,
}

impl Default for TwoPlayer {
    fn default() -> Self {
        Self { player0: true }
    }
}

impl Display for TwoPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Player {}", self.index())
    }
}

impl TwoPlayer {
    pub fn new(player0: bool) -> Self {
        Self { player0 }
    }

    pub fn next(&self) -> Self {
        Self {
            player0: !self.player0,
        }
    }

    pub fn next_mut(&mut self) {
        self.player0 = !self.player0
    }

    pub fn index(&self) -> usize {
        (!self.player0) as usize
    }

    pub fn last(&self) -> Self {
        self.next()
    }

    pub fn last_mut(&mut self) {
        self.next_mut()
    }
}

/// The Player type. Represents the players of a game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NPlayer {
    n_players: usize,
    current: usize,
}

impl Display for NPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Player {}", self.index())
    }
}

impl NPlayer {
    pub fn new(n_players: usize) -> Option<Self> {
        if n_players > 2 {
            Some(Self {
                n_players,
                current: 0,
            })
        } else {
            None
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
    pub fn index(&self) -> usize {
        self.current
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

//! Core logic for the Tic-Tac-Toe game.

use rand::Rng;
use std::fmt;

const BOARD_SIZE: usize = 3;

/// Represents a player, either X or O.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Player {
    X,
    O,
}

impl Player {
    /// Returns the other player.
    pub fn swap(&self) -> Self {
        match self {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Player::X => write!(f, "X"),
            Player::O => write!(f, "O"),
        }
    }
}

/// Represents a single cell on the board, which can be empty or occupied by a player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Empty,
    Occupied(Player),
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Cell::Empty => write!(f, "-"),
            Cell::Occupied(player) => write!(f, "{}", player),
        }
    }
}

/// Represents the overall state of the game.
#[derive(Debug, PartialEq, Eq)]
pub enum GameState {
    InProgress,
    Win(Player),
    Draw,
}

/// Represents the Tic-Tac-Toe game board and its state.
pub struct TicTacToe {
    board: [[Cell; BOARD_SIZE]; BOARD_SIZE],
    current_player: Player,
}

impl TicTacToe {
    /// Creates a new Tic-Tac-Toe game.
    ///
    /// The board is initialized to be empty, and the first player is chosen randomly.
    pub fn new() -> Self {
        let board = [[Cell::Empty; BOARD_SIZE]; BOARD_SIZE];
        let starting_player = if rand::thread_rng().gen_bool(0.5) {
            Player::X
        } else {
            Player::O
        };
        TicTacToe {
            board,
            current_player: starting_player,
        }
    }

    /// Returns the player whose turn it is.
    pub fn current_player(&self) -> Player {
        self.current_player
    }

    /// Displays the current state of the board to the console.
    pub fn show_board(&self) {
        for row in self.board {
            let row_str: Vec<String> = row.iter().map(|cell| cell.to_string()).collect();
            println!("{}", row_str.join(" "));
        }
        println!();
    }

    /// Attempts to place the current player's mark at the given spot.
    ///
    /// # Arguments
    /// * `row` - The 0-indexed row.
    /// * `col` - The 0-indexed column.
    ///
    /// # Errors
    /// Returns an error if the spot is out of bounds or already occupied.
    pub fn fix_spot(&mut self, row: usize, col: usize) -> Result<(), &'static str> {
        if row >= BOARD_SIZE || col >= BOARD_SIZE {
            return Err("Spot is out of bounds. Use numbers between 1 and 3.");
        }
        if self.board[row][col] != Cell::Empty {
            return Err("Spot is already taken.");
        }

        self.board[row][col] = Cell::Occupied(self.current_player);
        Ok(())
    }

    /// Checks the current game state to see if there is a win, a draw, or if it's still in progress.
    pub fn check_game_state(&self) -> GameState {
        // Check for a win for the player who just moved.
        if self.has_player_won(self.current_player) {
            return GameState::Win(self.current_player);
        }

        if self.is_board_filled() {
            return GameState::Draw;
        }

        GameState::InProgress
    }

    /// Switches the turn to the next player.
    pub fn swap_player_turn(&mut self) {
        self.current_player = self.current_player.swap();
    }

    /// Checks if the specified player has won the game.
    fn has_player_won(&self, player: Player) -> bool {
        let target_cell = Cell::Occupied(player);

        // Check rows and columns
        for i in 0..BOARD_SIZE {
            let row_win = (0..BOARD_SIZE).all(|j| self.board[i][j] == target_cell);
            let col_win = (0..BOARD_SIZE).all(|j| self.board[j][i] == target_cell);
            if row_win || col_win {
                return true;
            }
        }

        // Check diagonals
        let main_diag_win = (0..BOARD_SIZE).all(|i| self.board[i][i] == target_cell);
        let anti_diag_win =
            (0..BOARD_SIZE).all(|i| self.board[i][BOARD_SIZE - 1 - i] == target_cell);

        main_diag_win || anti_diag_win
    }

    /// Checks if the board is completely filled.
    fn is_board_filled(&self) -> bool {
        self.board
            .iter()
            .all(|row| row.iter().all(|&cell| cell != Cell::Empty))
    }
}

impl Default for TicTacToe {
    /// Provides a default way to create a new game.
    fn default() -> Self {
        Self::new()
    }
}

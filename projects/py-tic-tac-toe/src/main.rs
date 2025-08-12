//! A console-based Tic Tac Toe game.

use rand::Rng;
use std::fmt;
use std::io::{self, Write};

/// Represents a player, either X or O.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Player {
    X,
    O,
}

impl Player {
    /// Swaps the player turn.
    /// Returns `O` if the current player is `X`, and `X` if `O`.
    fn swap(&self) -> Self {
        match self {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }
}

/// Implement the `Display` trait for the `Player` enum to print 'X' or 'O'.
impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Player::X => write!(f, "X"),
            Player::O => write!(f, "O"),
        }
    }
}

/// Represents the state of a single cell on the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell {
    Player(Player),
    Empty,
}

/// Implement the `Display` trait for the `Cell` enum to print the player's symbol or '-'.
impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Cell::Player(player) => write!(f, "{}", player),
            Cell::Empty => write!(f, "-"),
        }
    }
}

/// Represents the Tic Tac Toe game state.
struct TicTacToe {
    board: [[Cell; 3]; 3],
}

impl TicTacToe {
    /// Creates a new, empty 3x3 Tic Tac Toe board.
    /// This corresponds to the Python `__init__` and `create_board` methods.
    fn new() -> Self {
        TicTacToe {
            board: [[Cell::Empty; 3]; 3],
        }
    }

    /// Places a player's symbol on the board at the given coordinates.
    /// Corresponds to the Python `fix_spot` method.
    fn fix_spot(&mut self, row: usize, col: usize, player: Player) {
        if row < 3 && col < 3 {
            self.board[row][col] = Cell::Player(player);
        }
    }

    /// Checks if the specified player has won the game.
    /// Corresponds to the Python `has_player_won` method.
    fn has_player_won(&self, player: Player) -> bool {
        let target_cell = Cell::Player(player);

        // Check rows
        for i in 0..3 {
            if self.board[i].iter().all(|&cell| cell == target_cell) {
                return true;
            }
        }

        // Check columns
        for i in 0..3 {
            if (0..3).all(|j| self.board[j][i] == target_cell) {
                return true;
            }
        }

        // Check diagonals
        if (0..3).all(|i| self.board[i][i] == target_cell) {
            return true;
        }
        if (0..3).all(|i| self.board[i][2 - i] == target_cell) {
            return true;
        }

        false
    }

    /// Checks if the board is completely filled.
    /// Corresponds to the Python `is_board_filled` method.
    fn is_board_filled(&self) -> bool {
        self.board
            .iter()
            .flatten()
            .all(|&cell| cell != Cell::Empty)
    }

    /// Starts and manages the main game loop.
    pub fn start(&mut self) {
        let mut current_player = get_random_first_player();
        let mut game_over = false;

        while !game_over {
            // Display the board using the Display trait implementation
            println!("{}", self);
            println!("Player {} turn", current_player);

            // Get user input for row and column
            print!("Enter row & column numbers to fix spot (e.g., 1 2): ");
            // Flush stdout to ensure the prompt is shown before reading input
            io::stdout().flush().unwrap();

            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                println!("\nError reading input. Please try again.");
                continue;
            }
            println!(); // Add a newline for spacing, like in the Python version

            // Parse user input into coordinates
            let coords: Vec<usize> = input
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();

            if coords.len() != 2 {
                println!("Invalid input. Please enter two numbers (1-3) separated by a space.");
                continue;
            }

            let (row, col) = (coords[0], coords[1]);

            // Check if input is in the valid range 1-3
            if !(1..=3).contains(&row) || !(1..=3).contains(&col) {
                println!("Invalid spot. Row and column must be between 1 and 3. Try again!");
                continue;
            }

            // Convert to 0-based index for internal board representation
            let row_idx = row - 1;
            let col_idx = col - 1;

            // Check if the spot is valid and not already taken
            if self.board[row_idx][col_idx] == Cell::Empty {
                self.fix_spot(row_idx, col_idx, current_player);

                // Check for win or draw conditions
                if self.has_player_won(current_player) {
                    println!("Player {} wins the game!", current_player);
                    game_over = true;
                } else if self.is_board_filled() {
                    println!("Match Draw!");
                    game_over = true;
                } else {
                    // Swap turns if the game is not over
                    current_player = current_player.swap();
                }
            } else {
                println!("That spot is already taken. Try again!");
            }
        }

        // Show the final board state
        println!("\nFinal Board:");
        println!("{}", self);
    }
}

/// Implement the `Display` trait for `TicTacToe` to provide a clean,
/// grid-based representation of the board.
/// Corresponds to the Python `show_board` method.
impl fmt::Display for TicTacToe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, row) in self.board.iter().enumerate() {
            writeln!(f, "{} {} {}", row[0], row[1], row[2])?;
        }
        Ok(())
    }
}

/// Randomly chooses which player goes first.
/// Corresponds to the Python `get_random_first_player` method.
fn get_random_first_player() -> Player {
    if rand::thread_rng().gen_bool(0.5) {
        Player::X
    } else {
        Player::O
    }
}

/// Main entry point of the application.
fn main() {
    // Corresponds to `if __name__ == '__main__':` block
    let mut tic_tac_toe = TicTacToe::new();
    tic_tac_toe.start();
}
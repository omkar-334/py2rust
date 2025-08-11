//! Tic Tac Toe
//! Repo owner: Md. Fahim Bin Amin (Original Python version)
//! Description: A console based Tic Tac Toe game, converted to idiomatic Rust.

use rand::Rng;
use std::fmt;
use std::io::{self, Write};

const BOARD_SIZE: usize = 3;

/// Represents a player, either X or O.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Player {
    X,
    O,
}

impl Player {
    /// Returns the other player.
    fn switch(self) -> Self {
        match self {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }
}

/// Implement the Display trait to allow printing the player's symbol.
impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Player::X => write!(f, "X"),
            Player::O => write!(f, "O"),
        }
    }
}

/// Represents the state of the game.
struct Game {
    /// The 3x3 board, where each cell is an Option<Player>.
    /// `None` represents an empty cell.
    board: [[Option<Player>; BOARD_SIZE]; BOARD_SIZE],
    /// The player whose turn it is.
    current_player: Player,
}

/// Represents possible errors when making a move.
#[derive(Debug)]
enum MoveError {
    OutOfBounds,
    SpotTaken,
}

impl Game {
    /// Creates a new game, initializing the board and randomly selecting a starting player.
    fn new() -> Self {
        let starting_player = if rand::thread_rng().gen_bool(0.5) {
            Player::X
        } else {
            Player::O
        };

        Game {
            board: [[None; BOARD_SIZE]; BOARD_SIZE],
            current_player: starting_player,
        }
    }

    /// Displays the current state of the board to the console.
    fn show_board(&self) {
        for row in self.board {
            let row_str: Vec<String> = row
                .iter()
                .map(|cell| match cell {
                    Some(player) => player.to_string(),
                    None => "-".to_string(),
                })
                .collect();
            println!("{}", row_str.join(" "));
        }
        println!();
    }

    /// Attempts to place the current player's mark at the given coordinates.
    ///
    /// # Arguments
    /// * `row` - The 0-indexed row.
    /// * `col` - The 0-indexed column.
    ///
    /// # Returns
    /// * `Ok(())` if the move was successful.
    /// * `Err(MoveError)` if the spot is out of bounds or already taken.
    fn make_move(&mut self, row: usize, col: usize) -> Result<(), MoveError> {
        if row >= BOARD_SIZE || col >= BOARD_SIZE {
            return Err(MoveError::OutOfBounds);
        }
        if self.board[row][col].is_some() {
            return Err(MoveError::SpotTaken);
        }
        self.board[row][col] = Some(self.current_player);
        Ok(())
    }

    /// Checks if the specified player has won the game.
    fn has_player_won(&self, player: Player) -> bool {
        // Check rows
        for i in 0..BOARD_SIZE {
            if self.board[i].iter().all(|&cell| cell == Some(player)) {
                return true;
            }
        }

        // Check columns
        for i in 0..BOARD_SIZE {
            if (0..BOARD_SIZE).all(|j| self.board[j][i] == Some(player)) {
                return true;
            }
        }

        // Check diagonals
        if (0..BOARD_SIZE).all(|i| self.board[i][i] == Some(player)) {
            return true;
        }
        if (0..BOARD_SIZE).all(|i| self.board[i][BOARD_SIZE - 1 - i] == Some(player)) {
            return true;
        }

        false
    }

    /// Checks if the board is completely filled, resulting in a draw.
    fn is_board_filled(&self) -> bool {
        self.board
            .iter()
            .all(|row| row.iter().all(|cell| cell.is_some()))
    }
}

/// Main game loop.
fn main() {
    let mut game = Game::new();

    loop {
        game.show_board();
        println!("Player {} turn", game.current_player);

        // Inner loop to handle user input until a valid move is entered.
        let (row, col) = loop {
            print!("Enter row & column numbers to fix spot (e.g., 1 1): ");
            // We must flush stdout to ensure the prompt is printed before we read input.
            io::stdout().flush().expect("Failed to flush stdout");

            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                println!("\nError: Failed to read line. Please try again.");
                continue;
            }

            let coords: Vec<Result<usize, _>> = input
                .trim()
                .split_whitespace()
                .map(|s| s.parse::<usize>())
                .collect();

            if coords.len() != 2 {
                println!("\nInvalid input: Please enter two numbers separated by a space.");
                continue;
            }

            match (&coords[0], &coords[1]) {
                (Ok(r), Ok(c)) => {
                    // The game uses 1-based indexing for user input, so we convert to 0-based.
                    if *r > 0 && *r <= BOARD_SIZE && *c > 0 && *c <= BOARD_SIZE {
                        break (*r - 1, *c - 1);
                    } else {
                        println!(
                            "\nInvalid input: Row and column must be between 1 and {}.",
                            BOARD_SIZE
                        );
                        continue;
                    }
                }
                _ => {
                    println!("\nInvalid input: Please enter valid numbers.");
                    continue;
                }
            }
        };
        println!();

        // Attempt to make the move and handle the result.
        match game.make_move(row, col) {
            Ok(_) => {
                if game.has_player_won(game.current_player) {
                    println!("Player {} wins the game!", game.current_player);
                    break;
                } else if game.is_board_filled() {
                    println!("Match Draw!");
                    break;
                } else {
                    game.current_player = game.current_player.switch();
                }
            }
            Err(MoveError::OutOfBounds) | Err(MoveError::SpotTaken) => {
                println!("Invalid spot. Try again!");
            }
        }
    }

    println!("Final board:");
    game.show_board();
}

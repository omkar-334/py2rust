//! Tic Tac Toe
//! Repo owner: Md. Fahim Bin Amin (Original Python version)
//! Description: A console based Tic Tac Toe game converted to idiomatic Rust.

use rand::Rng;
use std::fmt;
use std::io::{self, Write};

// --- Enums for Type Safety ---

/// Represents one of the two players.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Player {
    X,
    O,
}

impl Player {
    /// Returns the other player.
    fn swap(self) -> Self {
        match self {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }
}

/// Implement Display to easily print the player's symbol.
impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Player::X => write!(f, "X"),
            Player::O => write!(f, "O"),
        }
    }
}

/// Represents a single cell on the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell {
    Empty,
    Occupied(Player),
}

/// Implement Display to print the cell's content ('-', 'X', or 'O').
impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Cell::Empty => write!(f, "-"),
            Cell::Occupied(player) => write!(f, "{}", player),
        }
    }
}

/// Represents the state of the game after a move.
#[derive(Debug, PartialEq, Eq)]
enum GameState {
    Ongoing,
    Win(Player),
    Draw,
}

// --- Game Logic Struct ---

/// Represents the Tic Tac Toe game.
struct TicTacToe {
    board: [[Cell; 3]; 3],
    current_player: Player,
}

impl TicTacToe {
    /// Creates a new game instance.
    /// The board is initialized to be empty, and the first player is chosen randomly.
    pub fn new() -> Self {
        let starting_player = if rand::thread_rng().gen_bool(0.5) {
            Player::X
        } else {
            Player::O
        };

        TicTacToe {
            board: [[Cell::Empty; 3]; 3],
            current_player: starting_player,
        }
    }

    /// The main game loop, equivalent to the `start` method in Python.
    pub fn start(&mut self) {
        let mut game_over = false;

        while !game_over {
            self.show_board();
            println!("Player {} turn", self.current_player);

            // Loop until the user provides a valid and available spot.
            let (row, col) = loop {
                match self.get_user_input() {
                    Ok((r, c)) => {
                        if self.board[r][c] == Cell::Empty {
                            break (r, c);
                        } else {
                            println!("Invalid spot. That spot is already taken. Try again!");
                        }
                    }
                    Err(e) => println!("{}", e),
                }
            };

            self.fix_spot(row, col);

            match self.check_game_state() {
                GameState::Win(player) => {
                    println!("Player {} wins the game!", player);
                    game_over = true;
                }
                GameState::Draw => {
                    println!("Match Draw!");
                    game_over = true;
                }
                GameState::Ongoing => {
                    self.current_player = self.current_player.swap();
                }
            }
        }

        println!();
        self.show_board();
    }

    /// Prompts the user for input and parses it into 0-indexed board coordinates.
    fn get_user_input(&self) -> Result<(usize, usize), &'static str> {
        print!("Enter row & column numbers to fix spot (e.g., 1 2): ");
        // We need to flush stdout to make sure the prompt appears before read_line.
        io::stdout().flush().expect("Failed to flush stdout");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|_| "Failed to read line")?;

        let coords: Vec<usize> = input
            .trim()
            .split_whitespace()
            .filter_map(|s| s.parse().ok())
            .collect();

        if coords.len() != 2 {
            return Err("Invalid input. Please enter two numbers separated by a space.");
        }

        let row = coords[0];
        let col = coords[1];

        if !(1..=3).contains(&row) || !(1..=3).contains(&col) {
            return Err("Invalid spot. Row and column must be between 1 and 3.");
        }

        // Convert from 1-based user input to 0-based index.
        Ok((row - 1, col - 1))
    }

    /// Places the current player's mark on the board.
    fn fix_spot(&mut self, row: usize, col: usize) {
        self.board[row][col] = Cell::Occupied(self.current_player);
    }

    /// Checks the current state of the game (Win, Draw, or Ongoing).
    fn check_game_state(&self) -> GameState {
        if self.has_player_won(self.current_player) {
            return GameState::Win(self.current_player);
        }

        if self.is_board_filled() {
            return GameState::Draw;
        }

        GameState::Ongoing
    }

    /// Checks if the specified player has won the game.
    fn has_player_won(&self, player: Player) -> bool {
        let mark = Cell::Occupied(player);

        // Check rows and columns
        for i in 0..3 {
            if (self.board[i][0] == mark && self.board[i][1] == mark && self.board[i][2] == mark)
                || (self.board[0][i] == mark
                    && self.board[1][i] == mark
                    && self.board[2][i] == mark)
            {
                return true;
            }
        }

        // Check diagonals
        if (self.board[0][0] == mark && self.board[1][1] == mark && self.board[2][2] == mark)
            || (self.board[0][2] == mark && self.board[1][1] == mark && self.board[2][0] == mark)
        {
            return true;
        }

        false
    }

    /// Checks if the board is completely filled.
    fn is_board_filled(&self) -> bool {
        self.board
            .iter()
            .all(|row| row.iter().all(|&cell| cell != Cell::Empty))
    }

    /// Displays the current state of the board to the console.
    fn show_board(&self) {
        println!();
        for row in self.board {
            println!("{} {} {}", row[0], row[1], row[2]);
        }
        println!();
    }
}

/// Main entry point of the application.
fn main() {
    // Create a new game instance.
    let mut tic_tac_toe = TicTacToe::new();

    // Start the game.
    tic_tac_toe.start();
}

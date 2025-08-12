//! # Tic Tac Toe
//!
//! A console based Tic Tac Toe game converted from Python to Rust.
//!
//! ## Description
//! This program allows two players to play Tic Tac Toe in the console.
//! The first player is chosen randomly. Players take turns marking spots
//! on a 3x3 grid until one player wins or the board is full, resulting in a draw.

use rand::Rng;
use std::fmt;
use std::io::{self, Write};

/// Represents the two players in the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Player {
    X,
    O,
}

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

const BOARD_SIZE: usize = 3;

/// Represents the Tic Tac Toe game state and logic.
struct TicTacToe {
    board: [[Cell; BOARD_SIZE]; BOARD_SIZE],
}

impl TicTacToe {
    /// Creates a new Tic Tac Toe game with an empty board.
    pub fn new() -> Self {
        TicTacToe {
            board: [[Cell::Empty; BOARD_SIZE]; BOARD_SIZE],
        }
    }

    /// The main game loop. Manages player turns, input, and game state transitions.
    pub fn start(&mut self) {
        let mut current_player = self.get_random_first_player();
        let mut game_over = false;

        while !game_over {
            self.show_board();
            println!("Player {} turn", current_player);

            let (row, col) = self.get_player_input();

            if self.board[row][col] == Cell::Empty {
                self.fix_spot(row, col, current_player);

                if self.has_player_won(current_player) {
                    println!("\nPlayer {} wins the game!", current_player);
                    game_over = true;
                } else if self.is_board_filled() {
                    println!("\nMatch Draw!");
                    game_over = true;
                } else {
                    current_player = self.swap_player_turn(current_player);
                }
            } else {
                println!("Invalid spot. It's already taken. Try again!");
            }
            println!(); // Add a newline for better readability
        }

        println!("Final board:");
        self.show_board();
    }

    /// Prompts the user for input and returns valid, 0-indexed board coordinates.
    fn get_player_input(&self) -> (usize, usize) {
        loop {
            print!("Enter row & column numbers to fix spot (e.g., 1 1): ");
            // We need to flush stdout to ensure the prompt is shown before reading input.
            io::stdout().flush().unwrap();

            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                println!("Error reading input. Please try again.");
                continue;
            }

            let parts: Vec<&str> = input.trim().split_whitespace().collect();
            if parts.len() != 2 {
                println!("Invalid input. Please enter two numbers separated by a space.");
                continue;
            }

            let row_res = parts[0].parse::<usize>();
            let col_res = parts[1].parse::<usize>();

            match (row_res, col_res) {
                (Ok(r), Ok(c)) if r > 0 && c > 0 && r <= BOARD_SIZE && c <= BOARD_SIZE => {
                    // Convert from 1-based for user to 0-based for array index
                    return (r - 1, c - 1);
                }
                _ => {
                    println!("Invalid input. Please enter numbers between 1 and {}.", BOARD_SIZE);
                }
            }
        }
    }

    /// Randomly selects which player goes first.
    fn get_random_first_player(&self) -> Player {
        if rand::thread_rng().gen_bool(0.5) {
            Player::X
        } else {
            Player::O
        }
    }

    /// Places a player's mark on the board at the given coordinates.
    fn fix_spot(&mut self, row: usize, col: usize, player: Player) {
        self.board[row][col] = Cell::Occupied(player);
    }

    /// Checks if the given player has won the game by checking all win conditions.
    fn has_player_won(&self, player: Player) -> bool {
        let target = Cell::Occupied(player);

        // Check rows
        for i in 0..BOARD_SIZE {
            if self.board[i].iter().all(|&cell| cell == target) {
                return true;
            }
        }

        // Check columns
        for j in 0..BOARD_SIZE {
            if (0..BOARD_SIZE).all(|i| self.board[i][j] == target) {
                return true;
            }
        }

        // Check diagonals
        if (0..BOARD_SIZE).all(|i| self.board[i][i] == target) {
            return true;
        }
        if (0..BOARD_SIZE).all(|i| self.board[i][BOARD_SIZE - 1 - i] == target) {
            return true;
        }

        false
    }

    /// Checks if the board is completely filled.
    fn is_board_filled(&self) -> bool {
        self.board.iter().flatten().all(|&cell| cell != Cell::Empty)
    }

    /// Swaps the turn to the other player.
    fn swap_player_turn(&self, player: Player) -> Player {
        match player {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }

    /// Displays the current state of the board to the console.
    fn show_board(&self) {
        for row in self.board {
            let row_str: Vec<String> = row.iter().map(|cell| cell.to_string()).collect();
            println!("{}", row_str.join(" "));
        }
    }
}

fn main() {
    let mut tic_tac_toe = TicTacToe::new();
    tic_tac_toe.start();
}
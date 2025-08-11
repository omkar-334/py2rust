//! A console-based Tic Tac Toe game.

use anyhow::{anyhow, Result};
use std::io::{self, Write};
use tic_tac_toe_rust::{GameState, TicTacToe};

fn main() -> Result<()> {
    // Create a new game instance.
    let mut game = TicTacToe::new();

    // Main game loop.
    loop {
        game.show_board();
        let current_player = game.current_player();
        println!("Player {} turn", current_player);

        // Loop until valid user input is received and the move is made.
        loop {
            match get_user_input() {
                Ok((row, col)) => {
                    // Attempt to place the mark. If it fails, print the error and ask for input again.
                    if let Err(e) = game.fix_spot(row, col) {
                        println!("Invalid move: {}. Try again!", e);
                        continue;
                    }
                    // Valid move was made, so break the input loop.
                    break;
                }
                Err(e) => {
                    println!(
                        "Invalid input: {}. Please enter two numbers (e.g., '1 3').",
                        e
                    );
                }
            }
        }
        println!();

        // Check the game state after a successful move.
        match game.check_game_state() {
            GameState::Win(player) => {
                println!("Player {} wins the game!", player);
                break; // Exit the main game loop.
            }
            GameState::Draw => {
                println!("Match Draw!");
                break; // Exit the main game loop.
            }
            GameState::InProgress => {
                // If the game is still on, swap players and continue.
                game.swap_player_turn();
            }
        }
    }

    println!("Final Board:");
    game.show_board();

    Ok(())
}

/// Prompts the user for input and parses it into a 0-indexed (row, column) tuple.
fn get_user_input() -> Result<(usize, usize)> {
    print!("Enter row & column numbers to fix spot: ");
    // We must flush stdout to ensure the prompt is shown before we read input.
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    if parts.len() != 2 {
        return Err(anyhow!("Expected 2 numbers, but got {}", parts.len()));
    }

    // Parse numbers and convert from 1-based to 0-based index.
    let row: usize = parts[0].parse()?;
    let col: usize = parts[1].parse()?;

    if row == 0 || col == 0 {
        return Err(anyhow!("Row and column numbers cannot be zero."));
    }

    Ok((row - 1, col - 1))
}

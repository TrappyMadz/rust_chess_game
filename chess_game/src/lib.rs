// Contains all the code shared between the client and the server
// The program uses Cargo's default 'bin' directory feature, allowing the compilation of 2 separate programs from 1 project.
// This was done to avoid duplicating the message code between the server and the client. 

use serde::{Serialize, Deserialize};
use std::fmt;

// Variables
pub const SERVER_ADDRESS: &str = "127.0.0.1:8080";
pub const DEFAULT_BOARD: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

// Using .clone() instead of references for small enums to simplify ownership.
// For small enums, the performance impact is negligible, and it makes the code significantly easier to read.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum PlayerColor {
    White,
    Black,
}

// Having multiple message variants makes the communication logic easy to follow.
// The message can be treated differently according to their type.
// For the moves, FEN is used. It a format widely used for chess libraries.
#[derive(Serialize, Deserialize, Debug)]
pub enum GameMessage {
    Welcome(PlayerColor),
    GameState{ board : String, current_player: PlayerColor},
    MakeMove(String),
    ErrorMessage(String),
    EndMessage(String),
}

// Error handling - manually for a greater control
// Describe possible error types :
#[derive(Debug)]
pub enum ChessError {
    Network(std::io::Error),
    Protocol(serde_json::Error),
    Game(String),
}

// Display implementation to show the errors
impl fmt::Display for ChessError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ChessError::Network(error) => write!(f, "Network error : {}", error),
            ChessError::Protocol(error) => write!(f, "JSON error : {}", error),
            ChessError::Game(error) => write!(f, "Game logic error : {}", error),
        }
    }
}

// Standard error implementation, with default behavior
impl std::error::Error for ChessError {}

// If a network error happens, we want a Network chess error, not a std::io error.
impl From<std::io::Error> for ChessError {
    fn from(error: std::io::Error) -> Self {
        ChessError::Network(error)
    }
}

// Same with a serde error
impl From<serde_json::Error> for ChessError {
    fn from(error: serde_json::Error) -> Self {
        ChessError::Protocol(error)
    }
}


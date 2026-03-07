// Contains all the code shared between the client and the server
// The program uses Cargo's default 'bin' directory feature, allowing the compilation of 2 separate programs from 1 project.
// This was done to avoid duplicating the message code between the server and the client. 

use serde::{Serialize, Deserialize};

// Using .clone() instead of references for small enums to simplify ownership.
// For small enums, the performance impact is negligible, and it makes the code significantly easier to read.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum PlayerColor {
    White,
    Black,
}

// Having multiple message variants makes the communication logic easy to follow.
// The message can be treated differently according to their type.
#[derive(Serialize, Deserialize, Debug)]
pub enum GameMessage {
    Welcome(PlayerColor),
    GameState(String),
    MakeMove(String),
    ErrorMessage(String),
}


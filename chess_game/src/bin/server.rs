// The server code. The goal is to transmit messages and to generate logs and error messages.
// The server is only a server and nothing more.

use tokio::net::TcpListener;
use tokio::io::AsyncWriteExt;
use chess_game::{ChessError, GameMessage, PlayerColor};

// Variables
const SERVER_ADDRESS: &str = "127.0.0.1:8080";

// The server will wait for 2 players then start the game. It won't answer any additional players.
#[tokio::main]
async fn main() -> Result<(), ChessError> {
    let listener = TcpListener::bind(SERVER_ADDRESS).await?;

    println!("[SERVER] Started and listening on {}.", SERVER_ADDRESS);
    println!("[SERVER] Waiting for players...");

    // First player (White)
    let (mut white_socket, white_address) = listener.accept().await?;
    println!("[SERVER] Player 1 connected from {}. Assigned color : WHITE", white_address);

    // Sending a welcome message to confirm connection. \n to end the message
    let white_message = serde_json::to_string(&GameMessage::Welcome(PlayerColor::White))?;
    white_socket.write_all(format!("{}\n", white_message).as_bytes()).await?;

    // Second player (Black)
    let (mut black_socket, black_address) = listener.accept().await?;
    println!("[SERVER] Player 2 connected from {}. Assigned color : BLACK", black_address);

    // Sending a welcome message to confirm connection. \n to end the message
    let black_message = serde_json::to_string(&GameMessage::Welcome(PlayerColor::Black))?;
    black_socket.write_all(format!("{}\n", black_message).as_bytes()).await?;

    println!("[SERVER] Game ready to start.");

    Ok(())
}
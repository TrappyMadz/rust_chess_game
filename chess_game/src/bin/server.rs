// The server code. The goal is to transmit messages and to generate logs and error messages.
// The server is only a server and nothing more.

use tokio::net::TcpListener;
use tokio::io::AsyncWriteExt;
use std::str::FromStr;
use tokio::io::{AsyncBufReadExt, BufReader};
use chess::{Board, ChessMove, MoveGen};
use chess_game::{ChessError, GameMessage, PlayerColor, SERVER_ADDRESS, DEFAULT_BOARD};

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

    // Separating sockets into reading and writing to do both at the same time
    let (white_read, mut white_write) = white_socket.into_split();
    let (black_read, mut black_write) = black_socket.into_split();

    let mut white_reader = BufReader::new(white_read);
    let mut black_reader = BufReader::new(black_read);

    // Variables declaration
    let mut current_board_state = DEFAULT_BOARD.to_string();
    let mut current_turn = PlayerColor::White;

    // Gameplay loop
    loop {
        // Cloning has a negligible impact here, because it is done rarely (when players have done their move, so maximum once every ~30 seconds). 
        // The objects cloned are very small too.
        // The network speed will be the true bottleneck here.
        // Furthermore, cloning avoid the requirement of using lifetime everywhere, so coding is simpler.
        let state_message = GameMessage::GameState {
            board: current_board_state.clone(),
            current_player: current_turn.clone(),
        };

        // Sending state to both player
        let state_str = format!("{}\n", serde_json::to_string(&state_message)?);

        white_write.write_all(state_str.as_bytes()).await?;
        black_write.write_all(state_str.as_bytes()).await?;

        // Waiting for active player
        let mut line = String::new();
        let bytes_read = match current_turn {
            PlayerColor::White => white_reader.read_line(&mut line).await?,
            PlayerColor::Black => black_reader.read_line(&mut line).await?,
        };

        // If 0 byte where read, the player left
        if bytes_read == 0 {
            println!("[SERVER] Player {:?} disconnected. Game over.", current_turn);

            // Tell the other player
            let disconnect_message = GameMessage::ErrorMessage("The opponent disconnected. Game over.".to_string());
            let serialized_message = format!("{}\n", serde_json::to_string(&disconnect_message)?);

            // Sending to ALL players. We don't care about who disconnected or if the message is received, the server will close soon after that.
            // let _ and no "?" tell tokio : if the message is not received, ignore the error ! The player who didn't disconnect will get it before the server close.
            let _ = white_write.write_all(serialized_message.as_bytes()).await;
            let _ = black_write.write_all(serialized_message.as_bytes()).await;

            break;
        }

        // Decode message and treat the move
        let message: GameMessage = match serde_json::from_str(&line) {
            Ok(parsed_message) => parsed_message,
            Err(error) => {
                eprintln!("[SERVER] Failed to parse message from {:?}. Error : {}.", current_turn ,error);
                let err_message = GameMessage::ErrorMessage("Network error, server couldn't parse message. Try again.".to_string());
                if let Ok(err_str) = serde_json::to_string(&err_message) {
                    let formatted_err = format!("{}\n", err_str);
                    match current_turn {
                        PlayerColor::White => { let _ = white_write.write_all(formatted_err.as_bytes()).await; }
                        PlayerColor::Black => { let _ = black_write.write_all(formatted_err.as_bytes()).await; }
                    }
                }
                continue;
            },
        };

        // Chess logic
        if let GameMessage::MakeMove(move_str) = message {
            println!("[SERVER] Player {:?} wants to play : {}", current_turn, move_str);
            
            // Create board from FEN and validating the state BEFORE aplying the move
            let board = match Board::from_str(&current_board_state) {
                Ok(processed_board) => processed_board,
                Err(error) => {
                    // If the fen is not in a valid state, something went TERRIBLY wrong. This variable is not accessible by the players,
                    // Only the server and the crate can modify it.
                    // Something is really wrong, we stop the game.
                    eprintln!("[SERVER] CRITICAL ERROR : invalid FEN state ! ({}).",error);
                    let err_message = GameMessage::ErrorMessage("CRITICAL ERROR ! Corrupted board state, exiting game...".to_string());
                    let err_str = format!("{}\n", serde_json::to_string(&err_message)?);   
                    let _ = white_write.write_all(err_str.as_bytes()).await;
                    let _ = black_write.write_all(err_str.as_bytes()).await;
                    break;
                }
            };

            // Trying to parse the given move. If it's not a move, we tell the current player that whatever they wrote is not a move at all
            let parsed_move = match ChessMove::from_str(move_str.trim()) {
                Ok(processed_move) => processed_move,
                Err(_) => {
                    // Text is not a move
                    let err_message = GameMessage::ErrorMessage("Invalid format. exemple of valid format : 'f1e2'. Try again.".to_string());
                    let err_str = format!("{}\n", serde_json::to_string(&err_message)?);
                    match current_turn {
                        PlayerColor::White => { let _ = white_write.write_all(err_str.as_bytes()).await; }
                        PlayerColor::Black => { let _ = black_write.write_all(err_str.as_bytes()).await; }
                    }
                    continue; // We skip the rest of the loop
                }
            };

            // Is the move legal ?
            let mut is_legal = false;

            // Get all possible move (very fast, 30-40 possible move on average, theoretical maximum : 218). Furthermore, the chess crate is incredibly welle optimised.
            let iterable = MoveGen::new_legal(&board);
            for legal_move in iterable {
                if legal_move == parsed_move {
                    is_legal = true;
                    break;
                }
            }

            // Apply the result
            if is_legal {
                let new_board = board.make_move_new(parsed_move);

                // Win conditions :
                match new_board.status() {
                    chess::BoardStatus::Checkmate => {
                        println!("[SERVER] Game over. {:?} won by checkmate.", current_turn);
                        let win_message = GameMessage::EndMessage(format!("Checkmate ! {:?} win. Press Esc to quit.", current_turn));
                        if let Ok(win_str) = serde_json::to_string(&win_message) {
                            let formatted_win = format!("{}\n", win_str);
                            let _ = white_write.write_all(formatted_win.as_bytes()).await;
                            let _ = black_write.write_all(formatted_win.as_bytes()).await;
                        }
                        break;
                    },
                    chess::BoardStatus::Stalemate => {
                        println!("[SERVER] Game over. Stalemate (Draw).");
                        
                        let draw_message = GameMessage::EndMessage("Stalemate ! The game ends in a draw. Press Esc to quit".to_string());
                        if let Ok(draw_str) = serde_json::to_string(&draw_message) {
                            let formatted_draw = format!("{}\n", draw_str);
                            let _ = white_write.write_all(formatted_draw.as_bytes()).await;
                            let _ = black_write.write_all(formatted_draw.as_bytes()).await;
                        }
                        break;
                    },
                    chess::BoardStatus::Ongoing => {

                        
                        // get the new FEN
                        current_board_state = new_board.to_string();

                        // Turn change
                        current_turn = match current_turn {
                            PlayerColor::White => PlayerColor::Black,
                            PlayerColor::Black => PlayerColor::White,
                        }
                    }   
               
                };
            }
            else {
                // illegal move !
                let err_message = GameMessage::ErrorMessage("Illegal move for this board ! Try again.".to_string());
                let err_str = format!("{}\n", serde_json::to_string(&err_message)?);
                match current_turn {
                    PlayerColor::White => { let _ = white_write.write_all(err_str.as_bytes()).await; }
                    PlayerColor::Black => { let _ = black_write.write_all(err_str.as_bytes()).await; }
                }
            }


            
        }

    }

    Ok(())
}
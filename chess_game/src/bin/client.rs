// The client code. The goal is to follow the server's instructions, to transmit the will of the player to the server when it's their turn,
// and to draw the game state on screen.
// It is synchronous, meaning its execution is blocked while it is waiting for the server answer.

use std::net::TcpStream;
use std::io::{self, BufRead, BufReader, Write};
use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, Paragraph},
    Terminal,
    crossterm::{
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        event::{self, Event, KeyCode},
    },
};
use chess_game::{ChessError, GameMessage, PlayerColor, SERVER_ADDRESS, DEFAULT_BOARD};

// Struct to define the elements needed by the UI
// Avoid too many arguments in the ui function
struct AppState {
    current_board_state: String,
    my_color: Option<PlayerColor>,
    input: String,
    current_turn: PlayerColor,
    is_my_turn: bool,
    last_error: Option<String>,
    game_started: bool,
    is_game_finished: bool,
}

fn main() -> Result<(), ChessError> {

    // Connection
    println!("Connecting to {}...", SERVER_ADDRESS);
    let stream = TcpStream::connect(SERVER_ADDRESS)?;
    println!("Successfuly connected !");

    // Don't draw what the user is typing
    enable_raw_mode()?;

    // Open a screen only for the game
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    

    // Prepare for reading (it will read until next "\n")
    let mut reader = BufReader::new(&stream);
    let mut buffer = String::new();

    // Initializing state
    let mut state = AppState {
        current_board_state: DEFAULT_BOARD.to_string(),
        my_color: None,
        input: String::new(),
        current_turn: PlayerColor::White,
        is_my_turn: false,
        last_error: None,
        game_started: false,
        is_game_finished: false,
    };

    // to get the user input, we need to loop without blocking
    stream.set_nonblocking(true)?;

    // Drawing only if it is needed (performances)
    let mut needs_redraw = true;
    loop {
        // Get the user input (10ms to not block the display)
        if event::poll(std::time::Duration::from_millis(10))? 
            && let Event::Key(key) = event::read()? 
            && key.kind == event::KeyEventKind::Press 
        {
            needs_redraw = true;
            match key.code {
                // Adding character, deleting or entering :
                KeyCode::Char(character) => {
                    state.input.push(character);
                    state.last_error = None;
                },
                KeyCode::Backspace => { 
                    state.input.pop(); 
                    state.last_error = None;
                },
                KeyCode::Enter => {
                    if state.is_my_turn && !state.input.is_empty() {
                        let move_message = GameMessage::MakeMove(state.input.clone());
                        let mut serialized = serde_json::to_string(&move_message)?;

                        // End of message symbol
                        serialized.push('\n');

                        // using & to not consume the stream
                        let mut s = &stream;
                        s.write_all(serialized.as_bytes())?;

                        // force the stream to send the datas
                        s.flush()?;

                        // Cleaning up the input
                        state.input.clear();
                        state.is_my_turn = false;
                        state.last_error = None;
                    }
                }
                // Quitting
                KeyCode::Esc => break,
                _ => {},
            }
        }


        // Draw the ui only if something changed
        if needs_redraw {
            terminal.draw(|f| ui(f, &state))?;
            needs_redraw = false;
        }
        

        // Clearing the buffer before adding the new received messages.
        buffer.clear();

        // Waiting a message from server
        // If server is down, stopping the program
        // The loop is non-blocking, so the reader will crash. To avoid that, we need to ignore the errors
        match reader.read_line(&mut buffer) {
            // Server sent a message
            Ok(n) if n > 0 => {
                needs_redraw = true;

                // Updating variables when server send a message. 
                match serde_json::from_str::<GameMessage>(&buffer) {
                    Ok(message) => {
                        match message {
                            GameMessage::Welcome(color) => state.my_color = Some(color),
                            GameMessage::GameState { board, current_player} => {
                                state.current_board_state = board;
                                state.current_turn = current_player.clone();
                                state.is_my_turn = Some(current_player) == state.my_color;
                                state.game_started = true;
                            },
                            GameMessage::ErrorMessage(error) => {
                                state.last_error = Some(error);
                            }
                            GameMessage::EndMessage(result) => {
                                state.is_game_finished = true;
                                state.last_error = Some(result);
                            }
                            _ => {}
                        }
                    },
                    Err(e) => eprintln!("JSON Error: {}", e),
                }
            }
            // Server is closed
            Ok(0) => {
                if !state.is_game_finished {
                    state.last_error = Some("Connection with host lost, press Esc to exit.".to_string());
                }
                // If the game is finished, then we already displayed the result in the "error" field.
                needs_redraw = true;
            },
            // Nothing to read : "fake" error
            Err(ref error) if error.kind() == io::ErrorKind::WouldBlock => {
                // We do nothing
            }
            // A true error happened : return a Chess Error
            Err(error) => return Err(error.into()),
            // Empty buffer -> ignored
            _ => {}
        }
        
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

// ui function, to draw the state of the game
fn ui(f: &mut ratatui::Frame, state: &AppState) {
    // Get the size of the window
    let area = f.area();

    // Prepare the text
    let color_text = match &state.my_color {
        Some(color) => format!("{:?}", color),
        None => "Waiting for server...".to_string(),
    };

    // Prepare the board
    let board_visual = prepare_board(&state.current_board_state, &state.my_color);

    // Turn indicator
    let turn_indicator = if !state.game_started {
        "Waiting for opponent to connect...".to_string()
    }
    else if state.is_my_turn {
        ">>> YOUR TURN <<<".to_string()
    }
    else {
        format!("Waiting for {:?}...", state.current_turn)
    };

    // If there is an error, wi draw "white". There should not be errors here, it's just in case something goes wrong.
    // As ref because we only need to display it
    let mut display_text = format!(
        "Your color: {}\nStatus: {}\n\n{}\n\nYour move > {}", 
        color_text,
        turn_indicator,
        board_visual, 
        state.input
    );

    if let Some(error) = &state.last_error {
        display_text.push_str(&format!("\n\n{}", error));
    }

    // Rendring
    f.render_widget(
        Paragraph::new(display_text)
            .block(Block::default().title("Chess Game").borders(Borders::ALL)),
        area,
    );
}

// Tranform a FEN str into a chess board with unicode symbols
fn prepare_board(board: &str, my_color: &Option<PlayerColor>) -> String {
    let mut board_visual = String::new();

    // Getting the "position" part of the FEN text, or "" if there is an error
    // Here unwrap_or is used to ensure that something is returned, even if it's an empty string.
    let position_part = board.split(' ').next().unwrap_or("");

    // Each line will be a vector, that way we can manipulate them and flip the board for black
    let mut rows: Vec<&str> = position_part.split('/').collect();
    let is_black = matches!(my_color, Some(PlayerColor::Black));
    if is_black {
        rows.reverse();
    }

    // Prepare coordinates letters
    let letters = if is_black {
        "    h g f e d c b a\n"
    }
    else {
        "    a b c d e f g h\n"
    };

    // Prepare borders
    let top_border = "  ┌─────────────────┐\n";
    let bottom_border = "  └─────────────────┘\n";

    // Adding letters on top on top border
    board_visual.push_str(letters);
    board_visual.push_str(top_border);

    // Loop on rows to get indexes
    for (i, row) in rows.iter().enumerate() {
        // for white : 8-1 for black : 1-8
        let rank = if is_black { i + 1 } else { 8 - i };
        
        // Left border with number
        board_visual.push_str(&format!("{} │ ", rank));

        let mut characters: Vec<char> = row.chars().collect();
        if is_black {
            characters.reverse();
        }

        // Inside board
        for character in characters {
            if let Some(digit) = character.to_digit(10) {
                for _ in 0..digit { board_visual.push_str(". ");}
            } else {
                let symbol = match character {
                    'P' => "♙", 'R' => "♖", 'N' => "♘", 'B' => "♗", 'Q' => "♕", 'K' => "♔",
                    'p' => "♟", 'r' => "♜", 'n' => "♞", 'b' => "♝", 'q' => "♛", 'k' => "♚",
                    _ => "?",
                };
                board_visual.push_str(symbol);
                board_visual.push(' ');
            }
        }
        
        // Right border
        board_visual.push_str(&format!("│ {}\n", rank));
    }

    // Bottom border
    board_visual.push_str(bottom_border);
    board_visual.push_str(letters);

    board_visual
}
# Rust Multiplayer Chess

System Programming Project: Chess Game via TCP

## Project Overview

This project is a multiplayer chess game based on a Client/Server architecture. The goal is to allow two players to compete remotely over a TCP connection.

Technical Requirements Compliance:

- Server: Exclusively asynchronous, powered by the tokio runtime.
- Client: Synchronous execution managing the game loop and user inputs.
- Serialization: Data exchange handled via JSON format.
- Display: Terminal User Interface (TUI) implemented with the ratatui crate.

## Architecture and Design Choices

The project is structured into three distinct parts to ensure a clear separation of concerns:

- src/lib.rs: Contains shared data structures (GameMessage, PlayerColor) and constants (server address, default FEN). This ensures the client and server remain perfectly synchronized.
- src/bin/server.rs: An asynchronous server that manages game logic, validates moves via the chess crate, and enforces turn-based play.
- src/bin/client.rs: An synchronous client managing the ratatui interface. It utilizes a polling mechanism (event::poll) to maintain responsiveness while handling keyboard inputs.

Using cargo bin allows the project to maintain a single repository while providing two distinct executables sharing a common type library. The implementation of the State Pattern (AppState) on the client side ensures optimal performance by preventing unnecessary memory allocations during interface rendering.

## Dependencies

The project relies on the following ecosystem:

| Crate          | Configuration / Version                      |
| :------------- | :------------------------------------------- |
| **tokio**      | `{ version = "1", features = ["full"] }`     |
| **serde**      | `{ version = "1.0", features = ["derive"] }` |
| **serde_json** | `"1.0"`                                      |
| **chess**      | `"3.2.0"`                                    |
| **ratatui**    | `"0.30.0"`                                   |

> These versions are the ones recommended in their respective documentation.

## Getting Started

### Start the Server

Open a terminal at the project root and run:

```bash
cargo run --bin server
```

> The server listens by default on 127.0.0.1:8080.
> _You can cange that in lib.rs_

### Start the Clients

Open two separate terminals (one for each player) and run:

```bash
cargo run --bin client
```

## How to Play

The game uses the UCI (Universal Chess Interface) notation for move entry.

Type a command in the format [source square][destination square] (e.g., e2e4).

Coordinates and Interface:

- Borders: The board is surrounded by lettered columns (a-h) and numbered ranks (1-8).
- Board Flipping: If you are assigned the Black pieces, the board automatically flips to place your pieces at the bottom of the screen for better readability.

Special Moves:

- Standard Move: Type a command in the UCI format. _(ex: e2e4 to move a pawn or g1f3 for a knight.)_
- Castling: Move the King to its final destination square.
  - King-side: e1g1 (White) or e8g8 (Black).
  - Queen-side: e1c1 (White) or e8c8 (Black).
- En Passant: Enter the move for your pawn to its destination square (e.g., e5d6). The captured pawn is automatically removed by the server.
- Pawn Promotion: When a pawn reaches the last rank, append the letter of the desired piece at the end of the command:
  - q for a Queen (e.g., a7a8q)
  - r for a Rook
  - b for a Bishop
  - n for a Knight

Game Termination:
The server automatically detects Checkmate or Stalemate. A notification will appear on both players' screens. Press Esc at any time to safely exit the client.

> Note: The server implements a Fail-Fast policy. If a corrupted state is detected, the session is terminated to ensure the integrity of the game rules.

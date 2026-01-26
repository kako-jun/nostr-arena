//! Simple TUI example for nostr-arena

use nostr_arena::{Arena, ArenaConfig, ArenaEvent, StartMode};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GameState {
    score: i32,
    message: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("nostr-arena TUI Example");
    println!("=======================\n");

    let config = ArenaConfig::new("tui-example")
        .max_players(2)
        .start_mode(StartMode::Ready)
        .base_url("https://example.com");

    let arena: Arena<GameState> = Arena::new(config).await?;

    println!("Connecting to relays...");
    arena.connect().await?;
    println!("Connected! Public key: {}\n", arena.public_key());

    println!("Commands:");
    println!("  c - Create room");
    println!("  j <room_id> - Join room");
    println!("  r - Toggle ready");
    println!("  s <message> - Send state");
    println!("  l - List rooms");
    println!("  q - Quit\n");

    let mut ready = false;

    loop {
        // Check for events
        while let Some(event) = arena.try_recv().await {
            match event {
                ArenaEvent::PlayerJoin(player) => {
                    println!("[Event] Player joined: {}", &player.pubkey[..8]);
                }
                ArenaEvent::PlayerLeave(pubkey) => {
                    println!("[Event] Player left: {}", &pubkey[..8]);
                }
                ArenaEvent::PlayerState { pubkey, state } => {
                    println!("[Event] {} says: {}", &pubkey[..8], state.message);
                }
                ArenaEvent::AllReady => {
                    println!("[Event] All players ready!");
                }
                ArenaEvent::GameStart => {
                    println!("[Event] Game started!");
                }
                ArenaEvent::Error(msg) => {
                    println!("[Error] {}", msg);
                }
                _ => {}
            }
        }

        // Read input
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        let parts: Vec<&str> = input.splitn(2, ' ').collect();
        let cmd = parts[0];
        let arg = parts.get(1).copied().unwrap_or("");

        match cmd {
            "c" => {
                match arena.create().await {
                    Ok(url) => println!("Room created! URL: {}", url),
                    Err(e) => println!("Failed to create room: {}", e),
                }
            }
            "j" => {
                if arg.is_empty() {
                    println!("Usage: j <room_id>");
                    continue;
                }
                match arena.join(arg).await {
                    Ok(()) => println!("Joined room: {}", arg),
                    Err(e) => println!("Failed to join room: {}", e),
                }
            }
            "r" => {
                ready = !ready;
                match arena.send_ready(ready).await {
                    Ok(()) => println!("Ready: {}", ready),
                    Err(e) => println!("Failed to send ready: {}", e),
                }
            }
            "s" => {
                let state = GameState {
                    score: 0,
                    message: arg.to_string(),
                };
                match arena.send_state(&state).await {
                    Ok(()) => println!("State sent"),
                    Err(e) => println!("Failed to send state: {}", e),
                }
            }
            "l" => {
                match Arena::<GameState>::list_rooms(
                    "tui-example",
                    vec![
                        "wss://relay.damus.io".to_string(),
                        "wss://nos.lol".to_string(),
                    ],
                    None,
                    10,
                )
                .await
                {
                    Ok(rooms) => {
                        if rooms.is_empty() {
                            println!("No rooms found");
                        } else {
                            println!("Available rooms:");
                            for room in rooms {
                                println!(
                                    "  {} - {:?} ({}/{} players)",
                                    room.room_id, room.status, room.player_count, room.max_players
                                );
                            }
                        }
                    }
                    Err(e) => println!("Failed to list rooms: {}", e),
                }
            }
            "q" => {
                println!("Goodbye!");
                break;
            }
            _ => {
                println!("Unknown command: {}", cmd);
            }
        }
    }

    arena.disconnect().await?;
    Ok(())
}

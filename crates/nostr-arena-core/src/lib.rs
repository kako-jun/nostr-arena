//! # nostr-arena-core
//!
//! Nostr-based real-time multiplayer game arena. No server required.
//!
//! ## Features
//!
//! - **Room Discovery**: Find available game rooms
//! - **Presence Tracking**: Track multiple players in a room
//! - **Start Modes**: Auto, Ready, Countdown, or Host-controlled
//! - **Game State Sync**: Real-time state synchronization
//! - **Reconnection**: Automatic reconnection support
//!
//! ## Example
//!
//! ```rust,ignore
//! use nostr_arena_core::{Arena, ArenaConfig, ArenaEvent};
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Clone, Serialize, Deserialize)]
//! struct GameState {
//!     score: i32,
//!     position: (f32, f32),
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = ArenaConfig::new("my-game")
//!         .max_players(4)
//!         .start_mode(StartMode::Ready);
//!
//!     let arena: Arena<GameState> = Arena::new(config).await?;
//!     arena.connect().await?;
//!
//!     // Create a room
//!     let url = arena.create().await?;
//!     println!("Share this URL: {}", url);
//!
//!     // Wait for events
//!     while let Some(event) = arena.recv().await {
//!         match event {
//!             ArenaEvent::PlayerJoin(player) => {
//!                 println!("Player joined: {}", player.pubkey);
//!             }
//!             ArenaEvent::GameStart => {
//!                 println!("Game started!");
//!             }
//!             ArenaEvent::PlayerState { pubkey, state } => {
//!                 println!("Player {} score: {}", pubkey, state.score);
//!             }
//!             _ => {}
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```

pub mod arena;
pub mod client;
pub mod error;
pub mod types;

pub use arena::{Arena, ArenaEvent};
pub use client::NostrClient;
pub use error::{ArenaError, Result};
pub use types::*;

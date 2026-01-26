//! # nostr-arena
//!
//! Nostr-based real-time multiplayer game arena. No server required.
//!
//! ## Features
//!
//! - **Room Discovery**: Find available game rooms
//! - **Presence Tracking**: Track multiple players in a room
//! - **Start Modes**: Auto, Ready, Countdown, or Host-controlled
//! - **Game State Sync**: Real-time state synchronization
//! - **QR Code**: Generate QR codes for room sharing
//!
//! ## Example
//!
//! ```rust,ignore
//! use nostr_arena::{Arena, ArenaConfig, ArenaEvent, StartMode};
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
//!     let url = arena.create().await?;
//!     println!("Share this URL: {}", url);
//!
//!     while let Some(event) = arena.recv().await {
//!         match event {
//!             ArenaEvent::PlayerJoin(player) => {
//!                 println!("Player joined: {}", player.pubkey);
//!             }
//!             ArenaEvent::GameStart => {
//!                 println!("Game started!");
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
pub mod qr;
pub mod spawn;
pub mod time;
pub mod types;

#[cfg(test)]
mod tests;

pub use arena::{Arena, ArenaEvent};
pub use client::NostrClient;
pub use error::{ArenaError, Result};
pub use qr::{QrOptions, generate_qr_data_url, generate_qr_svg};
pub use types::*;

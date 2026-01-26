//! Error types for nostr-arena

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ArenaError {
    #[error("Not connected to relays")]
    NotConnected,

    #[error("Room not found")]
    RoomNotFound,

    #[error("Room expired")]
    RoomExpired,

    #[error("Room is full")]
    RoomFull,

    #[error("Room deleted")]
    RoomDeleted,

    #[error("Invalid room data: {0}")]
    InvalidRoomData(String),

    #[error("Operation timed out")]
    Timeout,

    #[error("Not authorized: {0}")]
    NotAuthorized(String),

    #[error("Already in room")]
    AlreadyInRoom,

    #[error("Not in room")]
    NotInRoom,

    #[error("Nostr error: {0}")]
    Nostr(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, ArenaError>;

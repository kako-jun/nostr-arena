//! Type definitions for nostr-arena

use serde::{Deserialize, Serialize};

/// Nostr event kinds used by the library
pub mod kinds {
    /// Replaceable event for room metadata (NIP-78)
    pub const ROOM: u16 = 30078;
    /// Ephemeral event for game state (not stored by relays)
    pub const EPHEMERAL: u16 = 25000;
}

/// Room status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RoomStatus {
    #[default]
    Idle,
    Creating,
    Waiting,
    Joining,
    Ready,
    Playing,
    Finished,
    Deleted,
}

/// Start mode for game initiation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StartMode {
    /// Game starts immediately when all players join
    #[default]
    Auto,
    /// Game starts when all players send ready signal
    Ready,
    /// Game starts after countdown when all players ready
    Countdown,
    /// Host manually starts the game
    Host,
}

/// Arena configuration
#[derive(Debug, Clone)]
pub struct ArenaConfig {
    /// Unique identifier for the game (e.g., "sasso", "tetris")
    pub game_id: String,
    /// Nostr relay URLs
    pub relays: Vec<String>,
    /// Room expiration time in ms (0 = never, default: 0)
    pub room_expiry: u64,
    /// Heartbeat interval in ms (default: 3000)
    pub heartbeat_interval: u64,
    /// Disconnect threshold in ms (default: 10000)
    pub disconnect_threshold: u64,
    /// State update throttle in ms (default: 100)
    pub state_throttle: u64,
    /// Join timeout in ms (default: 30000)
    pub join_timeout: u64,
    /// Maximum players (default: 2)
    pub max_players: usize,
    /// Start mode (default: Auto)
    pub start_mode: StartMode,
    /// Countdown seconds for Countdown mode (default: 3)
    pub countdown_seconds: u32,
    /// Base URL for room URLs
    pub base_url: Option<String>,
}

impl Default for ArenaConfig {
    fn default() -> Self {
        Self {
            game_id: String::new(),
            relays: vec![
                "wss://relay.damus.io".to_string(),
                "wss://nos.lol".to_string(),
                "wss://relay.nostr.band".to_string(),
            ],
            room_expiry: 0, // Never expire by default
            heartbeat_interval: 3000,
            disconnect_threshold: 10000,
            state_throttle: 100,
            join_timeout: 30000,
            max_players: 2,
            start_mode: StartMode::Auto,
            countdown_seconds: 3,
            base_url: None,
        }
    }
}

impl ArenaConfig {
    pub fn new(game_id: impl Into<String>) -> Self {
        Self {
            game_id: game_id.into(),
            ..Default::default()
        }
    }

    pub fn relays(mut self, relays: Vec<String>) -> Self {
        self.relays = relays;
        self
    }

    pub fn room_expiry(mut self, ms: u64) -> Self {
        self.room_expiry = ms;
        self
    }

    pub fn max_players(mut self, n: usize) -> Self {
        self.max_players = n;
        self
    }

    pub fn start_mode(mut self, mode: StartMode) -> Self {
        self.start_mode = mode;
        self
    }

    pub fn countdown_seconds(mut self, secs: u32) -> Self {
        self.countdown_seconds = secs;
        self
    }

    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }
}

/// Room state (game-agnostic)
#[derive(Debug, Clone, Default)]
pub struct RoomState {
    pub room_id: Option<String>,
    pub status: RoomStatus,
    pub is_host: bool,
    pub seed: u64,
    pub created_at: Option<u64>,
    pub expires_at: Option<u64>,
    pub rematch_requested: bool,
}

/// Player presence information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerPresence {
    pub pubkey: String,
    pub joined_at: u64,
    pub last_seen: u64,
    pub ready: bool,
}

/// Room info for discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomInfo {
    pub room_id: String,
    pub game_id: String,
    pub status: RoomStatus,
    pub host_pubkey: String,
    pub player_count: usize,
    pub max_players: usize,
    pub created_at: u64,
    pub expires_at: Option<u64>,
    pub seed: u64,
}

/// Opponent state with generic game state
#[derive(Debug, Clone)]
pub struct OpponentState<T> {
    pub public_key: String,
    pub game_state: Option<T>,
    pub is_connected: bool,
    pub last_heartbeat: u64,
    pub rematch_requested: bool,
}

impl<T> OpponentState<T> {
    pub fn new(public_key: String) -> Self {
        Self {
            public_key,
            game_state: None,
            is_connected: true,
            last_heartbeat: now_ms(),
            rematch_requested: false,
        }
    }
}

// Event content types

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum EventContent {
    Room(RoomEventContent),
    Join(JoinEventContent),
    State(StateEventContent),
    GameOver(GameOverEventContent),
    Rematch(RematchEventContent),
    Heartbeat(HeartbeatEventContent),
    Ready(ReadyEventContent),
    GameStart(GameStartEventContent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomEventContent {
    pub status: RoomStatus,
    pub seed: u64,
    pub host_pubkey: String,
    pub max_players: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<u64>,
    #[serde(default)]
    pub players: Vec<PlayerPresence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinEventContent {
    pub player_pubkey: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateEventContent {
    pub game_state: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameOverEventContent {
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_score: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub winner: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RematchEventContent {
    pub action: RematchAction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_seed: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RematchAction {
    Request,
    Accept,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatEventContent {
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadyEventContent {
    pub ready: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStartEventContent {}

/// Generate room tag from game ID and room ID
pub fn create_room_tag(game_id: &str, room_id: &str) -> String {
    format!("{game_id}-{room_id}")
}

/// Generate a random seed
pub fn generate_seed() -> u64 {
    use rand::Rng;
    rand::thread_rng().r#gen()
}

/// Generate a unique room ID (6 chars)
pub fn generate_room_id() -> String {
    use rand::Rng;
    const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..6)
        .map(|_| CHARS[rng.gen_range(0..CHARS.len())] as char)
        .collect()
}

/// Current time in milliseconds
pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

//! Python bindings for nostr-arena

use nostr_arena_core::{
    Arena as CoreArena, ArenaConfig as CoreConfig, ArenaEvent as CoreEvent,
    PlayerPresence as CorePlayerPresence, RoomInfo as CoreRoomInfo, RoomStatus, StartMode,
};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Arena configuration
#[pyclass]
#[derive(Clone)]
pub struct ArenaConfig {
    inner: CoreConfig,
}

#[pymethods]
impl ArenaConfig {
    #[new]
    fn new(game_id: &str) -> Self {
        Self {
            inner: CoreConfig::new(game_id),
        }
    }

    fn relays(mut self_: PyRefMut<'_, Self>, relays: Vec<String>) -> PyRefMut<'_, Self> {
        self_.inner = self_.inner.clone().relays(relays);
        self_
    }

    fn room_expiry(mut self_: PyRefMut<'_, Self>, ms: u64) -> PyRefMut<'_, Self> {
        self_.inner = self_.inner.clone().room_expiry(ms);
        self_
    }

    fn max_players(mut self_: PyRefMut<'_, Self>, n: usize) -> PyRefMut<'_, Self> {
        self_.inner = self_.inner.clone().max_players(n);
        self_
    }

    fn start_mode(mut self_: PyRefMut<'_, Self>, mode: &str) -> PyRefMut<'_, Self> {
        let mode = match mode {
            "auto" => StartMode::Auto,
            "ready" => StartMode::Ready,
            "countdown" => StartMode::Countdown,
            "host" => StartMode::Host,
            _ => StartMode::Auto,
        };
        self_.inner = self_.inner.clone().start_mode(mode);
        self_
    }

    fn countdown_seconds(mut self_: PyRefMut<'_, Self>, secs: u32) -> PyRefMut<'_, Self> {
        self_.inner = self_.inner.clone().countdown_seconds(secs);
        self_
    }

    fn base_url(mut self_: PyRefMut<'_, Self>, url: &str) -> PyRefMut<'_, Self> {
        self_.inner = self_.inner.clone().base_url(url);
        self_
    }
}

/// Player presence information
#[pyclass]
#[derive(Clone)]
pub struct PlayerPresence {
    #[pyo3(get)]
    pub pubkey: String,
    #[pyo3(get)]
    pub joined_at: u64,
    #[pyo3(get)]
    pub last_seen: u64,
    #[pyo3(get)]
    pub ready: bool,
}

impl From<CorePlayerPresence> for PlayerPresence {
    fn from(p: CorePlayerPresence) -> Self {
        Self {
            pubkey: p.pubkey,
            joined_at: p.joined_at,
            last_seen: p.last_seen,
            ready: p.ready,
        }
    }
}

/// Room information for discovery
#[pyclass]
#[derive(Clone)]
pub struct RoomInfo {
    #[pyo3(get)]
    pub room_id: String,
    #[pyo3(get)]
    pub game_id: String,
    #[pyo3(get)]
    pub status: String,
    #[pyo3(get)]
    pub host_pubkey: String,
    #[pyo3(get)]
    pub player_count: usize,
    #[pyo3(get)]
    pub max_players: usize,
    #[pyo3(get)]
    pub created_at: u64,
    #[pyo3(get)]
    pub expires_at: Option<u64>,
    #[pyo3(get)]
    pub seed: u64,
}

impl From<CoreRoomInfo> for RoomInfo {
    fn from(r: CoreRoomInfo) -> Self {
        let status = match r.status {
            RoomStatus::Idle => "idle",
            RoomStatus::Creating => "creating",
            RoomStatus::Waiting => "waiting",
            RoomStatus::Joining => "joining",
            RoomStatus::Ready => "ready",
            RoomStatus::Playing => "playing",
            RoomStatus::Finished => "finished",
            RoomStatus::Deleted => "deleted",
        };
        Self {
            room_id: r.room_id,
            game_id: r.game_id,
            status: status.to_string(),
            host_pubkey: r.host_pubkey,
            player_count: r.player_count,
            max_players: r.max_players,
            created_at: r.created_at,
            expires_at: r.expires_at,
            seed: r.seed,
        }
    }
}

/// Arena event
#[pyclass]
#[derive(Clone)]
pub struct ArenaEvent {
    #[pyo3(get)]
    pub event_type: String,
    #[pyo3(get)]
    pub pubkey: Option<String>,
    #[pyo3(get)]
    pub player: Option<PlayerPresence>,
    #[pyo3(get)]
    pub state: Option<String>,  // JSON string
    #[pyo3(get)]
    pub reason: Option<String>,
    #[pyo3(get)]
    pub final_score: Option<i64>,
    #[pyo3(get)]
    pub seed: Option<u64>,
    #[pyo3(get)]
    pub seconds: Option<u32>,
    #[pyo3(get)]
    pub message: Option<String>,
}

impl<T: serde::Serialize> From<CoreEvent<T>> for ArenaEvent {
    fn from(e: CoreEvent<T>) -> Self {
        match e {
            CoreEvent::PlayerJoin(p) => Self {
                event_type: "player_join".to_string(),
                player: Some(p.into()),
                pubkey: None,
                state: None,
                reason: None,
                final_score: None,
                seed: None,
                seconds: None,
                message: None,
            },
            CoreEvent::PlayerLeave(pubkey) => Self {
                event_type: "player_leave".to_string(),
                pubkey: Some(pubkey),
                player: None,
                state: None,
                reason: None,
                final_score: None,
                seed: None,
                seconds: None,
                message: None,
            },
            CoreEvent::PlayerState { pubkey, state } => Self {
                event_type: "player_state".to_string(),
                pubkey: Some(pubkey),
                state: serde_json::to_string(&state).ok(),
                player: None,
                reason: None,
                final_score: None,
                seed: None,
                seconds: None,
                message: None,
            },
            CoreEvent::PlayerDisconnect(pubkey) => Self {
                event_type: "player_disconnect".to_string(),
                pubkey: Some(pubkey),
                player: None,
                state: None,
                reason: None,
                final_score: None,
                seed: None,
                seconds: None,
                message: None,
            },
            CoreEvent::PlayerGameOver { pubkey, reason, final_score } => Self {
                event_type: "player_game_over".to_string(),
                pubkey: Some(pubkey),
                reason: Some(reason),
                final_score,
                player: None,
                state: None,
                seed: None,
                seconds: None,
                message: None,
            },
            CoreEvent::RematchRequested(pubkey) => Self {
                event_type: "rematch_requested".to_string(),
                pubkey: Some(pubkey),
                player: None,
                state: None,
                reason: None,
                final_score: None,
                seed: None,
                seconds: None,
                message: None,
            },
            CoreEvent::RematchStart(seed) => Self {
                event_type: "rematch_start".to_string(),
                seed: Some(seed),
                pubkey: None,
                player: None,
                state: None,
                reason: None,
                final_score: None,
                seconds: None,
                message: None,
            },
            CoreEvent::AllReady => Self {
                event_type: "all_ready".to_string(),
                pubkey: None,
                player: None,
                state: None,
                reason: None,
                final_score: None,
                seed: None,
                seconds: None,
                message: None,
            },
            CoreEvent::CountdownStart(seconds) => Self {
                event_type: "countdown_start".to_string(),
                seconds: Some(seconds),
                pubkey: None,
                player: None,
                state: None,
                reason: None,
                final_score: None,
                seed: None,
                message: None,
            },
            CoreEvent::CountdownTick(remaining) => Self {
                event_type: "countdown_tick".to_string(),
                seconds: Some(remaining),
                pubkey: None,
                player: None,
                state: None,
                reason: None,
                final_score: None,
                seed: None,
                message: None,
            },
            CoreEvent::GameStart => Self {
                event_type: "game_start".to_string(),
                pubkey: None,
                player: None,
                state: None,
                reason: None,
                final_score: None,
                seed: None,
                seconds: None,
                message: None,
            },
            CoreEvent::Error(message) => Self {
                event_type: "error".to_string(),
                message: Some(message),
                pubkey: None,
                player: None,
                state: None,
                reason: None,
                final_score: None,
                seed: None,
                seconds: None,
            },
        }
    }
}

/// Arena - Main game room manager
#[pyclass]
pub struct Arena {
    inner: Arc<RwLock<CoreArena<serde_json::Value>>>,
    runtime: tokio::runtime::Runtime,
}

#[pymethods]
impl Arena {
    #[new]
    fn new(config: &ArenaConfig) -> PyResult<Self> {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        let inner = runtime
            .block_on(CoreArena::new(config.inner.clone()))
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        Ok(Self {
            inner: Arc::new(RwLock::new(inner)),
            runtime,
        })
    }

    /// Get public key
    fn public_key(&self) -> String {
        self.runtime.block_on(async {
            self.inner.read().await.public_key()
        })
    }

    /// Connect to relays
    fn connect(&self) -> PyResult<()> {
        self.runtime.block_on(async {
            self.inner
                .read()
                .await
                .connect()
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
        })
    }

    /// Disconnect from relays
    fn disconnect(&self) -> PyResult<()> {
        self.runtime.block_on(async {
            self.inner
                .read()
                .await
                .disconnect()
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
        })
    }

    /// Check if connected
    fn is_connected(&self) -> bool {
        self.runtime.block_on(async {
            self.inner.read().await.is_connected().await
        })
    }

    /// Create a new room
    fn create(&self) -> PyResult<String> {
        self.runtime.block_on(async {
            self.inner
                .read()
                .await
                .create()
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
        })
    }

    /// Join an existing room
    fn join(&self, room_id: &str) -> PyResult<()> {
        self.runtime.block_on(async {
            self.inner
                .read()
                .await
                .join(room_id)
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
        })
    }

    /// Leave the current room
    fn leave(&self) -> PyResult<()> {
        self.runtime.block_on(async {
            self.inner
                .read()
                .await
                .leave()
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
        })
    }

    /// Delete the room (host only)
    fn delete_room(&self) -> PyResult<()> {
        self.runtime.block_on(async {
            self.inner
                .read()
                .await
                .delete_room()
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
        })
    }

    /// Send game state (JSON string)
    fn send_state(&self, state_json: &str) -> PyResult<()> {
        let state: serde_json::Value = serde_json::from_str(state_json)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

        self.runtime.block_on(async {
            self.inner
                .read()
                .await
                .send_state(&state)
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
        })
    }

    /// Send game over
    fn send_game_over(&self, reason: &str, final_score: Option<i64>) -> PyResult<()> {
        self.runtime.block_on(async {
            self.inner
                .read()
                .await
                .send_game_over(reason, final_score)
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
        })
    }

    /// Request rematch
    fn request_rematch(&self) -> PyResult<()> {
        self.runtime.block_on(async {
            self.inner
                .read()
                .await
                .request_rematch()
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
        })
    }

    /// Accept rematch
    fn accept_rematch(&self) -> PyResult<()> {
        self.runtime.block_on(async {
            self.inner
                .read()
                .await
                .accept_rematch()
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
        })
    }

    /// Send ready signal
    fn send_ready(&self, ready: bool) -> PyResult<()> {
        self.runtime.block_on(async {
            self.inner
                .read()
                .await
                .send_ready(ready)
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
        })
    }

    /// Start game (host only)
    fn start_game(&self) -> PyResult<()> {
        self.runtime.block_on(async {
            self.inner
                .read()
                .await
                .start_game()
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
        })
    }

    /// Get room URL
    fn get_room_url(&self) -> Option<String> {
        self.runtime.block_on(async {
            self.inner.read().await.get_room_url().await
        })
    }

    /// Get current players
    fn players(&self) -> Vec<PlayerPresence> {
        self.runtime.block_on(async {
            self.inner
                .read()
                .await
                .players()
                .await
                .into_iter()
                .map(|p| p.into())
                .collect()
        })
    }

    /// Get player count
    fn player_count(&self) -> usize {
        self.runtime.block_on(async {
            self.inner.read().await.player_count().await
        })
    }

    /// Poll for next event (non-blocking)
    fn try_recv(&self) -> Option<ArenaEvent> {
        self.runtime.block_on(async {
            self.inner
                .read()
                .await
                .try_recv()
                .await
                .map(|e| e.into())
        })
    }

    /// List available rooms
    #[staticmethod]
    fn list_rooms(
        py: Python<'_>,
        game_id: &str,
        relays: Vec<String>,
        status: Option<&str>,
        limit: usize,
    ) -> PyResult<Vec<RoomInfo>> {
        let status_filter = status.map(|s| match s {
            "waiting" => RoomStatus::Waiting,
            "playing" => RoomStatus::Playing,
            "finished" => RoomStatus::Finished,
            _ => RoomStatus::Waiting,
        });

        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        let game_id = game_id.to_string();
        runtime.block_on(async move {
            CoreArena::<serde_json::Value>::list_rooms(&game_id, relays, status_filter, limit)
                .await
                .map(|rooms| rooms.into_iter().map(|r| r.into()).collect())
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
        })
    }
}

/// Python module
#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<ArenaConfig>()?;
    m.add_class::<Arena>()?;
    m.add_class::<PlayerPresence>()?;
    m.add_class::<RoomInfo>()?;
    m.add_class::<ArenaEvent>()?;
    Ok(())
}

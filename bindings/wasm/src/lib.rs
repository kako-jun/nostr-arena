//! WebAssembly bindings for nostr-arena

use nostr_arena_core::{
    Arena as CoreArena, ArenaConfig as CoreConfig, ArenaEvent as CoreEvent,
    PlayerPresence, RoomStatus, StartMode,
};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
}

/// Arena configuration
#[wasm_bindgen]
pub struct ArenaConfig {
    inner: CoreConfig,
}

#[wasm_bindgen]
impl ArenaConfig {
    #[wasm_bindgen(constructor)]
    pub fn new(game_id: &str) -> Self {
        Self {
            inner: CoreConfig::new(game_id),
        }
    }

    #[wasm_bindgen(js_name = setRelays)]
    pub fn set_relays(mut self, relays: Vec<String>) -> Self {
        self.inner = self.inner.relays(relays);
        self
    }

    #[wasm_bindgen(js_name = setRoomExpiry)]
    pub fn set_room_expiry(mut self, ms: u64) -> Self {
        self.inner = self.inner.room_expiry(ms);
        self
    }

    #[wasm_bindgen(js_name = setMaxPlayers)]
    pub fn set_max_players(mut self, n: usize) -> Self {
        self.inner = self.inner.max_players(n);
        self
    }

    #[wasm_bindgen(js_name = setStartMode)]
    pub fn set_start_mode(mut self, mode: &str) -> Self {
        let mode = match mode {
            "auto" => StartMode::Auto,
            "ready" => StartMode::Ready,
            "countdown" => StartMode::Countdown,
            "host" => StartMode::Host,
            _ => StartMode::Auto,
        };
        self.inner = self.inner.start_mode(mode);
        self
    }

    #[wasm_bindgen(js_name = setCountdownSeconds)]
    pub fn set_countdown_seconds(mut self, secs: u32) -> Self {
        self.inner = self.inner.countdown_seconds(secs);
        self
    }

    #[wasm_bindgen(js_name = setBaseUrl)]
    pub fn set_base_url(mut self, url: &str) -> Self {
        self.inner = self.inner.base_url(url);
        self
    }
}

/// Arena - Main game room manager
#[wasm_bindgen]
pub struct Arena {
    inner: CoreArena<serde_json::Value>,
}

#[wasm_bindgen]
impl Arena {
    /// Create a new Arena instance
    #[wasm_bindgen(js_name = init)]
    pub async fn init(config: ArenaConfig) -> Result<Arena, JsValue> {
        let inner = CoreArena::new(config.inner)
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(Self { inner })
    }

    /// Get public key
    #[wasm_bindgen(js_name = publicKey)]
    pub fn public_key(&self) -> String {
        self.inner.public_key()
    }

    /// Connect to relays
    pub async fn connect(&self) -> Result<(), JsValue> {
        self.inner
            .connect()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Disconnect from relays
    pub async fn disconnect(&self) -> Result<(), JsValue> {
        self.inner
            .disconnect()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Check if connected
    #[wasm_bindgen(js_name = isConnected)]
    pub async fn is_connected(&self) -> bool {
        self.inner.is_connected().await
    }

    /// Create a new room
    pub async fn create(&self) -> Result<String, JsValue> {
        self.inner
            .create()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Join an existing room
    pub async fn join(&self, room_id: &str) -> Result<(), JsValue> {
        self.inner
            .join(room_id)
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Leave the current room
    pub async fn leave(&self) -> Result<(), JsValue> {
        self.inner
            .leave()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Delete the room (host only)
    #[wasm_bindgen(js_name = deleteRoom)]
    pub async fn delete_room(&self) -> Result<(), JsValue> {
        self.inner
            .delete_room()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Send game state
    #[wasm_bindgen(js_name = sendState)]
    pub async fn send_state(&self, state: JsValue) -> Result<(), JsValue> {
        let state: serde_json::Value = serde_wasm_bindgen::from_value(state)?;
        self.inner
            .send_state(&state)
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Send game over
    #[wasm_bindgen(js_name = sendGameOver)]
    pub async fn send_game_over(&self, reason: &str, final_score: Option<i64>) -> Result<(), JsValue> {
        self.inner
            .send_game_over(reason, final_score)
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Request rematch
    #[wasm_bindgen(js_name = requestRematch)]
    pub async fn request_rematch(&self) -> Result<(), JsValue> {
        self.inner
            .request_rematch()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Accept rematch
    #[wasm_bindgen(js_name = acceptRematch)]
    pub async fn accept_rematch(&self) -> Result<(), JsValue> {
        self.inner
            .accept_rematch()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Send ready signal
    #[wasm_bindgen(js_name = sendReady)]
    pub async fn send_ready(&self, ready: bool) -> Result<(), JsValue> {
        self.inner
            .send_ready(ready)
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Start game (host only)
    #[wasm_bindgen(js_name = startGame)]
    pub async fn start_game(&self) -> Result<(), JsValue> {
        self.inner
            .start_game()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get room URL
    #[wasm_bindgen(js_name = getRoomUrl)]
    pub async fn get_room_url(&self) -> Option<String> {
        self.inner.get_room_url().await
    }

    /// Get current players
    pub async fn players(&self) -> Result<JsValue, JsValue> {
        let players = self.inner.players().await;
        serde_wasm_bindgen::to_value(&players).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get player count
    #[wasm_bindgen(js_name = playerCount)]
    pub async fn player_count(&self) -> usize {
        self.inner.player_count().await
    }

    /// Poll for next event (non-blocking)
    #[wasm_bindgen(js_name = tryRecv)]
    pub async fn try_recv(&self) -> Result<JsValue, JsValue> {
        match self.inner.try_recv().await {
            Some(event) => {
                let js_event = event_to_js(event)?;
                Ok(js_event)
            }
            None => Ok(JsValue::NULL),
        }
    }

    /// List available rooms (static method)
    #[wasm_bindgen(js_name = listRooms)]
    pub async fn list_rooms(
        game_id: &str,
        relays: Vec<String>,
        status: Option<String>,
        limit: usize,
    ) -> Result<JsValue, JsValue> {
        let status_filter = status.as_deref().map(|s| match s {
            "waiting" => RoomStatus::Waiting,
            "playing" => RoomStatus::Playing,
            "finished" => RoomStatus::Finished,
            _ => RoomStatus::Waiting,
        });

        let rooms = CoreArena::<serde_json::Value>::list_rooms(game_id, relays, status_filter, limit)
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        serde_wasm_bindgen::to_value(&rooms).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

fn event_to_js(event: CoreEvent<serde_json::Value>) -> Result<JsValue, JsValue> {
    #[derive(Serialize)]
    #[serde(tag = "type", rename_all = "camelCase")]
    enum JsEvent {
        PlayerJoin { player: PlayerPresence },
        PlayerLeave { pubkey: String },
        PlayerState { pubkey: String, state: serde_json::Value },
        PlayerDisconnect { pubkey: String },
        PlayerGameOver { pubkey: String, reason: String, final_score: Option<i64> },
        RematchRequested { pubkey: String },
        RematchStart { seed: u64 },
        AllReady,
        CountdownStart { seconds: u32 },
        CountdownTick { remaining: u32 },
        GameStart,
        Error { message: String },
    }

    let js_event = match event {
        CoreEvent::PlayerJoin(player) => JsEvent::PlayerJoin { player },
        CoreEvent::PlayerLeave(pubkey) => JsEvent::PlayerLeave { pubkey },
        CoreEvent::PlayerState { pubkey, state } => JsEvent::PlayerState { pubkey, state },
        CoreEvent::PlayerDisconnect(pubkey) => JsEvent::PlayerDisconnect { pubkey },
        CoreEvent::PlayerGameOver { pubkey, reason, final_score } => {
            JsEvent::PlayerGameOver { pubkey, reason, final_score }
        }
        CoreEvent::RematchRequested(pubkey) => JsEvent::RematchRequested { pubkey },
        CoreEvent::RematchStart(seed) => JsEvent::RematchStart { seed },
        CoreEvent::AllReady => JsEvent::AllReady,
        CoreEvent::CountdownStart(seconds) => JsEvent::CountdownStart { seconds },
        CoreEvent::CountdownTick(remaining) => JsEvent::CountdownTick { remaining },
        CoreEvent::GameStart => JsEvent::GameStart,
        CoreEvent::Error(message) => JsEvent::Error { message },
    };

    serde_wasm_bindgen::to_value(&js_event).map_err(|e| JsValue::from_str(&e.to_string()))
}

//! Arena - Main game room management

use crate::client::NostrClient;
use crate::error::{ArenaError, Result};
use crate::types::*;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, Duration};
use tracing::{info, warn};

/// Arena events emitted to the application
#[derive(Debug, Clone)]
pub enum ArenaEvent<T> {
    /// Player joined the room
    PlayerJoin(PlayerPresence),
    /// Player left the room
    PlayerLeave(String),
    /// Player state updated
    PlayerState { pubkey: String, state: T },
    /// Player disconnected (heartbeat timeout)
    PlayerDisconnect(String),
    /// Player sent game over
    PlayerGameOver {
        pubkey: String,
        reason: String,
        final_score: Option<i64>,
    },
    /// Player requested rematch
    RematchRequested(String),
    /// Rematch accepted, new game starting
    RematchStart(u64),
    /// All players ready
    AllReady,
    /// Countdown started
    CountdownStart(u32),
    /// Countdown tick
    CountdownTick(u32),
    /// Game started
    GameStart,
    /// Error occurred
    Error(String),
}

/// Arena - Manages a multiplayer game room over Nostr
pub struct Arena<T> {
    config: ArenaConfig,
    client: Arc<NostrClient>,
    room_state: Arc<RwLock<RoomState>>,
    players: Arc<RwLock<HashMap<String, PlayerPresence>>>,
    player_states: Arc<RwLock<HashMap<String, T>>>,
    event_tx: mpsc::Sender<ArenaEvent<T>>,
    event_rx: Arc<RwLock<mpsc::Receiver<ArenaEvent<T>>>>,
    last_state_update: Arc<RwLock<u64>>,
    _marker: PhantomData<T>,
}

impl<T> Arena<T>
where
    T: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    /// Create a new Arena
    pub async fn new(config: ArenaConfig) -> Result<Self> {
        let client = NostrClient::new(config.relays.clone()).await?;
        let (event_tx, event_rx) = mpsc::channel(100);

        Ok(Self {
            config,
            client: Arc::new(client),
            room_state: Arc::new(RwLock::new(RoomState::default())),
            players: Arc::new(RwLock::new(HashMap::new())),
            player_states: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            event_rx: Arc::new(RwLock::new(event_rx)),
            last_state_update: Arc::new(RwLock::new(0)),
            _marker: PhantomData,
        })
    }

    /// Get the public key
    pub fn public_key(&self) -> String {
        self.client.public_key()
    }

    /// Get current room state
    pub async fn room_state(&self) -> RoomState {
        self.room_state.read().await.clone()
    }

    /// Get current players
    pub async fn players(&self) -> Vec<PlayerPresence> {
        self.players.read().await.values().cloned().collect()
    }

    /// Get player count
    pub async fn player_count(&self) -> usize {
        self.players.read().await.len()
    }

    /// Receive next event (non-blocking)
    pub async fn try_recv(&self) -> Option<ArenaEvent<T>> {
        self.event_rx.write().await.try_recv().ok()
    }

    /// Receive next event (blocking)
    pub async fn recv(&self) -> Option<ArenaEvent<T>> {
        self.event_rx.write().await.recv().await
    }

    /// Connect to relays
    pub async fn connect(&self) -> Result<()> {
        self.client.connect().await
    }

    /// Disconnect from relays
    pub async fn disconnect(&self) -> Result<()> {
        self.client.disconnect().await
    }

    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        self.client.is_connected().await
    }

    // =========================================================================
    // Room Discovery (Static)
    // =========================================================================

    /// List available rooms
    pub async fn list_rooms(
        game_id: &str,
        relays: Vec<String>,
        status_filter: Option<RoomStatus>,
        limit: usize,
    ) -> Result<Vec<RoomInfo>> {
        let client = NostrClient::new(relays).await?;
        client.connect().await?;

        let events = client.fetch_rooms(game_id, limit * 2).await?;
        let now = now_ms();

        let mut rooms = Vec::new();
        for event in events {
            if let Ok(content) = serde_json::from_str::<RoomEventContent>(&event.content) {
                // Skip deleted rooms
                if content.status == RoomStatus::Deleted {
                    continue;
                }

                // Skip expired rooms
                if let Some(expires_at) = content.expires_at {
                    if now > expires_at {
                        continue;
                    }
                }

                // Apply status filter
                if let Some(filter) = status_filter {
                    if content.status != filter {
                        continue;
                    }
                }

                // Extract room_id from d tag
                let room_id = event
                    .tags
                    .iter()
                    .find_map(|tag| {
                        if tag.kind() == nostr_sdk::TagKind::SingleLetter(nostr_sdk::SingleLetterTag::lowercase(nostr_sdk::Alphabet::D)) {
                            tag.content().map(|s| {
                                s.strip_prefix(&format!("{game_id}-"))
                                    .unwrap_or(s)
                                    .to_string()
                            })
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default();

                rooms.push(RoomInfo {
                    room_id,
                    game_id: game_id.to_string(),
                    status: content.status,
                    host_pubkey: content.host_pubkey,
                    player_count: content.players.len(),
                    max_players: content.max_players,
                    created_at: event.created_at.as_u64() * 1000,
                    expires_at: content.expires_at,
                    seed: content.seed,
                });
            }
        }

        rooms.truncate(limit);
        client.disconnect().await?;
        Ok(rooms)
    }

    // =========================================================================
    // Room Management
    // =========================================================================

    /// Create a new room
    pub async fn create(&self) -> Result<String> {
        if !self.client.is_connected().await {
            self.client.connect().await?;
        }

        let room_id = generate_room_id();
        let seed = generate_seed();
        let created_at = now_ms();
        let expires_at = if self.config.room_expiry > 0 {
            Some(created_at + self.config.room_expiry)
        } else {
            None
        };

        // Update local state
        {
            let mut state = self.room_state.write().await;
            state.room_id = Some(room_id.clone());
            state.status = RoomStatus::Creating;
            state.is_host = true;
            state.seed = seed;
            state.created_at = Some(created_at);
            state.expires_at = expires_at;
        }

        // Add self to players
        {
            let mut players = self.players.write().await;
            players.insert(
                self.public_key(),
                PlayerPresence {
                    pubkey: self.public_key(),
                    joined_at: created_at,
                    last_seen: created_at,
                    ready: false,
                },
            );
        }

        // Publish room event
        let room_tag = create_room_tag(&self.config.game_id, &room_id);
        let content = RoomEventContent {
            status: RoomStatus::Waiting,
            seed,
            host_pubkey: self.public_key(),
            max_players: self.config.max_players,
            expires_at,
            players: self.players.read().await.values().cloned().collect(),
        };

        self.client
            .publish_room(
                &room_tag,
                &self.config.game_id,
                &serde_json::to_string(&content)?,
            )
            .await?;

        // Update status
        {
            let mut state = self.room_state.write().await;
            state.status = RoomStatus::Waiting;
        }

        // Start subscription and heartbeat
        self.start_room_subscription(&room_id).await?;
        self.start_heartbeat().await;
        self.start_presence_update().await;

        // Generate room URL
        let url = if let Some(base) = &self.config.base_url {
            format!("{base}/battle/{room_id}")
        } else {
            format!("/battle/{room_id}")
        };

        info!("Created room: {}", room_id);
        Ok(url)
    }

    /// Join an existing room
    pub async fn join(&self, room_id: &str) -> Result<()> {
        if !self.client.is_connected().await {
            self.client.connect().await?;
        }

        let room_tag = create_room_tag(&self.config.game_id, room_id);

        // Fetch room info
        let event = self
            .client
            .fetch_room(&room_tag)
            .await?
            .ok_or(ArenaError::RoomNotFound)?;

        let content: RoomEventContent =
            serde_json::from_str(&event.content).map_err(|e| ArenaError::InvalidRoomData(e.to_string()))?;

        // Check room status
        if content.status == RoomStatus::Deleted {
            return Err(ArenaError::RoomDeleted);
        }

        // Check expiry
        if let Some(expires_at) = content.expires_at {
            if now_ms() > expires_at {
                return Err(ArenaError::RoomExpired);
            }
        }

        // Check player count
        if content.players.len() >= content.max_players {
            return Err(ArenaError::RoomFull);
        }

        let created_at = event.created_at.as_u64() * 1000;
        let now = now_ms();

        // Update local state
        {
            let mut state = self.room_state.write().await;
            state.room_id = Some(room_id.to_string());
            state.status = RoomStatus::Joining;
            state.is_host = false;
            state.seed = content.seed;
            state.created_at = Some(created_at);
            state.expires_at = content.expires_at;
        }

        // Add existing players
        {
            let mut players = self.players.write().await;
            for p in content.players {
                players.insert(p.pubkey.clone(), p);
            }
            // Add self
            players.insert(
                self.public_key(),
                PlayerPresence {
                    pubkey: self.public_key(),
                    joined_at: now,
                    last_seen: now,
                    ready: false,
                },
            );
        }

        // Send join event
        let join_content = serde_json::to_string(&EventContent::Join(JoinEventContent {
            player_pubkey: self.public_key(),
        }))?;

        self.client.publish_ephemeral(&room_tag, &join_content).await?;

        // Start subscription
        self.start_room_subscription(room_id).await?;

        // Update status
        {
            let mut state = self.room_state.write().await;
            state.status = RoomStatus::Ready;
        }

        // Start heartbeat
        self.start_heartbeat().await;

        // Send additional join events for reliability
        let client = self.client.clone();
        let tag = room_tag.clone();
        let content = join_content.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(500)).await;
            let _ = client.publish_ephemeral(&tag, &content).await;
            tokio::time::sleep(Duration::from_millis(1000)).await;
            let _ = client.publish_ephemeral(&tag, &content).await;
        });

        // Check if we should auto-start
        self.check_auto_start().await;

        info!("Joined room: {}", room_id);
        Ok(())
    }

    /// Leave the current room
    pub async fn leave(&self) -> Result<()> {
        let mut state = self.room_state.write().await;
        state.room_id = None;
        state.status = RoomStatus::Idle;
        state.is_host = false;
        self.players.write().await.clear();
        self.player_states.write().await.clear();
        Ok(())
    }

    /// Delete the room (host only)
    pub async fn delete_room(&self) -> Result<()> {
        let state = self.room_state.read().await;
        if !state.is_host {
            return Err(ArenaError::NotAuthorized("Only host can delete room".to_string()));
        }

        let room_id = state.room_id.as_ref().ok_or(ArenaError::NotInRoom)?;
        let room_tag = create_room_tag(&self.config.game_id, room_id);

        let content = RoomEventContent {
            status: RoomStatus::Deleted,
            seed: state.seed,
            host_pubkey: self.public_key(),
            max_players: self.config.max_players,
            expires_at: state.expires_at,
            players: vec![],
        };

        self.client
            .publish_room(
                &room_tag,
                &self.config.game_id,
                &serde_json::to_string(&content)?,
            )
            .await?;

        drop(state);
        self.leave().await?;
        info!("Deleted room");
        Ok(())
    }

    /// Reconnect to a room (e.g., after page refresh or connection drop)
    pub async fn reconnect(&self, room_id: &str) -> Result<()> {
        // First, leave any current room cleanly
        self.leave().await?;

        // Then join the specified room
        self.join(room_id).await?;

        info!("Reconnected to room: {}", room_id);
        Ok(())
    }

    // =========================================================================
    // Game State
    // =========================================================================

    /// Send game state to other players (throttled)
    pub async fn send_state(&self, state: &T) -> Result<()> {
        let now = now_ms();
        let last = *self.last_state_update.read().await;

        if now - last < self.config.state_throttle {
            return Ok(());
        }

        *self.last_state_update.write().await = now;

        let room_state = self.room_state.read().await;
        let room_id = room_state.room_id.as_ref().ok_or(ArenaError::NotInRoom)?;
        let room_tag = create_room_tag(&self.config.game_id, room_id);

        let content = serde_json::to_string(&EventContent::State(StateEventContent {
            game_state: serde_json::to_value(state)?,
        }))?;

        self.client.publish_ephemeral(&room_tag, &content).await?;
        Ok(())
    }

    /// Send game over event
    pub async fn send_game_over(&self, reason: &str, final_score: Option<i64>) -> Result<()> {
        let room_state = self.room_state.read().await;
        let room_id = room_state.room_id.as_ref().ok_or(ArenaError::NotInRoom)?;
        let room_tag = create_room_tag(&self.config.game_id, room_id);

        let content = serde_json::to_string(&EventContent::GameOver(GameOverEventContent {
            reason: reason.to_string(),
            final_score,
            winner: None,
        }))?;

        self.client.publish_ephemeral(&room_tag, &content).await?;

        drop(room_state);
        let mut state = self.room_state.write().await;
        state.status = RoomStatus::Finished;

        Ok(())
    }

    /// Request a rematch
    pub async fn request_rematch(&self) -> Result<()> {
        let room_state = self.room_state.read().await;
        if room_state.status != RoomStatus::Finished {
            return Ok(());
        }

        let room_id = room_state.room_id.as_ref().ok_or(ArenaError::NotInRoom)?;
        let room_tag = create_room_tag(&self.config.game_id, room_id);

        let content = serde_json::to_string(&EventContent::Rematch(RematchEventContent {
            action: RematchAction::Request,
            new_seed: None,
        }))?;

        self.client.publish_ephemeral(&room_tag, &content).await?;

        drop(room_state);
        let mut state = self.room_state.write().await;
        state.rematch_requested = true;

        Ok(())
    }

    /// Accept a rematch
    pub async fn accept_rematch(&self) -> Result<()> {
        let room_state = self.room_state.read().await;
        let room_id = room_state.room_id.as_ref().ok_or(ArenaError::NotInRoom)?;
        let room_tag = create_room_tag(&self.config.game_id, room_id);

        let new_seed = generate_seed();
        let content = serde_json::to_string(&EventContent::Rematch(RematchEventContent {
            action: RematchAction::Accept,
            new_seed: Some(new_seed),
        }))?;

        self.client.publish_ephemeral(&room_tag, &content).await?;

        drop(room_state);
        self.reset_for_rematch(new_seed).await;

        Ok(())
    }

    // =========================================================================
    // Start Mode
    // =========================================================================

    /// Send ready signal (for Ready/Countdown modes)
    pub async fn send_ready(&self, ready: bool) -> Result<()> {
        let room_state = self.room_state.read().await;
        let room_id = room_state.room_id.as_ref().ok_or(ArenaError::NotInRoom)?;
        let room_tag = create_room_tag(&self.config.game_id, room_id);

        let content = serde_json::to_string(&EventContent::Ready(ReadyEventContent { ready }))?;

        self.client.publish_ephemeral(&room_tag, &content).await?;

        // Update self ready status
        let mut players = self.players.write().await;
        if let Some(p) = players.get_mut(&self.public_key()) {
            p.ready = ready;
        }

        drop(room_state);
        drop(players);
        self.check_all_ready().await;

        Ok(())
    }

    /// Start the game (for Host mode, host only)
    pub async fn start_game(&self) -> Result<()> {
        let room_state = self.room_state.read().await;
        if !room_state.is_host {
            return Err(ArenaError::NotAuthorized("Only host can start game".to_string()));
        }

        let room_id = room_state.room_id.as_ref().ok_or(ArenaError::NotInRoom)?;
        let room_tag = create_room_tag(&self.config.game_id, room_id);

        let content = serde_json::to_string(&EventContent::GameStart(GameStartEventContent {}))?;

        self.client.publish_ephemeral(&room_tag, &content).await?;

        drop(room_state);
        let mut state = self.room_state.write().await;
        state.status = RoomStatus::Playing;

        let _ = self.event_tx.send(ArenaEvent::GameStart).await;

        Ok(())
    }

    // =========================================================================
    // QR Code / URL
    // =========================================================================

    /// Get room URL
    pub async fn get_room_url(&self) -> Option<String> {
        let state = self.room_state.read().await;
        let room_id = state.room_id.as_ref()?;

        if let Some(base) = &self.config.base_url {
            Some(format!("{base}/battle/{room_id}"))
        } else {
            Some(format!("/battle/{room_id}"))
        }
    }

    /// Get room QR code as SVG
    pub async fn get_room_qr_svg(&self, options: Option<crate::qr::QrOptions>) -> Option<String> {
        let url = self.get_room_url().await?;
        crate::qr::generate_qr_svg(&url, &options.unwrap_or_default()).ok()
    }

    /// Get room QR code as data URL
    pub async fn get_room_qr_data_url(&self, options: Option<crate::qr::QrOptions>) -> Option<String> {
        let url = self.get_room_url().await?;
        crate::qr::generate_qr_data_url(&url, &options.unwrap_or_default()).ok()
    }

    // =========================================================================
    // Private: Event Handling
    // =========================================================================

    async fn start_room_subscription(&self, room_id: &str) -> Result<()> {
        let room_tag = create_room_tag(&self.config.game_id, room_id);
        let my_pubkey = self.public_key();
        let players = self.players.clone();
        let player_states = self.player_states.clone();
        let room_state = self.room_state.clone();
        let event_tx = self.event_tx.clone();
        let config = self.config.clone();

        self.client
            .subscribe_room(&room_tag, move |event| {
                // Skip own events
                if event.pubkey.to_hex() == my_pubkey {
                    return;
                }

                let pubkey = event.pubkey.to_hex();

                // Parse content
                if let Ok(content) = serde_json::from_str::<EventContent>(&event.content) {
                    let players = players.clone();
                    let player_states = player_states.clone();
                    let room_state = room_state.clone();
                    let event_tx = event_tx.clone();
                    let config = config.clone();

                    tokio::spawn(async move {
                        match content {
                            EventContent::Join(join) => {
                                let now = now_ms();
                                let presence = PlayerPresence {
                                    pubkey: join.player_pubkey.clone(),
                                    joined_at: now,
                                    last_seen: now,
                                    ready: false,
                                };

                                players.write().await.insert(join.player_pubkey.clone(), presence.clone());

                                let _ = event_tx.send(ArenaEvent::PlayerJoin(presence)).await;

                                // Check auto-start
                                if config.start_mode == StartMode::Auto {
                                    let player_count = players.read().await.len();
                                    if player_count >= config.max_players {
                                        let mut state = room_state.write().await;
                                        state.status = RoomStatus::Playing;
                                        let _ = event_tx.send(ArenaEvent::GameStart).await;
                                    }
                                }
                            }

                            EventContent::State(state_event) => {
                                // Update last_seen
                                if let Some(p) = players.write().await.get_mut(&pubkey) {
                                    p.last_seen = now_ms();
                                }

                                if let Ok(state) = serde_json::from_value::<T>(state_event.game_state) {
                                    player_states.write().await.insert(pubkey.clone(), state.clone());
                                    let _ = event_tx.send(ArenaEvent::PlayerState { pubkey, state }).await;
                                }
                            }

                            EventContent::Heartbeat(hb) => {
                                if let Some(p) = players.write().await.get_mut(&pubkey) {
                                    p.last_seen = hb.timestamp;
                                }
                            }

                            EventContent::GameOver(go) => {
                                let _ = event_tx
                                    .send(ArenaEvent::PlayerGameOver {
                                        pubkey,
                                        reason: go.reason,
                                        final_score: go.final_score,
                                    })
                                    .await;

                                room_state.write().await.status = RoomStatus::Finished;
                            }

                            EventContent::Rematch(rm) => {
                                match rm.action {
                                    RematchAction::Request => {
                                        let _ = event_tx.send(ArenaEvent::RematchRequested(pubkey)).await;
                                    }
                                    RematchAction::Accept => {
                                        if let Some(new_seed) = rm.new_seed {
                                            let mut state = room_state.write().await;
                                            state.seed = new_seed;
                                            state.status = RoomStatus::Ready;
                                            state.rematch_requested = false;
                                            let _ = event_tx.send(ArenaEvent::RematchStart(new_seed)).await;
                                        }
                                    }
                                }
                            }

                            EventContent::Ready(r) => {
                                if let Some(p) = players.write().await.get_mut(&pubkey) {
                                    p.ready = r.ready;
                                }

                                // Check if all ready
                                let all_ready = players.read().await.values().all(|p| p.ready);
                                if all_ready {
                                    let _ = event_tx.send(ArenaEvent::AllReady).await;

                                    match config.start_mode {
                                        StartMode::Ready => {
                                            room_state.write().await.status = RoomStatus::Playing;
                                            let _ = event_tx.send(ArenaEvent::GameStart).await;
                                        }
                                        StartMode::Countdown => {
                                            let secs = config.countdown_seconds;
                                            let _ = event_tx.send(ArenaEvent::CountdownStart(secs)).await;

                                            // Spawn countdown task
                                            let event_tx_clone = event_tx.clone();
                                            let room_state_clone = room_state.clone();
                                            tokio::spawn(async move {
                                                for remaining in (1..=secs).rev() {
                                                    tokio::time::sleep(Duration::from_secs(1)).await;
                                                    let _ = event_tx_clone.send(ArenaEvent::CountdownTick(remaining - 1)).await;
                                                }
                                                room_state_clone.write().await.status = RoomStatus::Playing;
                                                let _ = event_tx_clone.send(ArenaEvent::GameStart).await;
                                            });
                                        }
                                        _ => {}
                                    }
                                }
                            }

                            EventContent::GameStart(_) => {
                                room_state.write().await.status = RoomStatus::Playing;
                                let _ = event_tx.send(ArenaEvent::GameStart).await;
                            }

                            EventContent::Room(_) => {
                                // Room metadata update - usually ignored in ephemeral subscription
                            }
                        }
                    });
                }
            })
            .await?;

        Ok(())
    }

    async fn start_heartbeat(&self) {
        let client = self.client.clone();
        let room_state = self.room_state.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_millis(config.heartbeat_interval));

            loop {
                ticker.tick().await;

                let state = room_state.read().await;
                if let Some(room_id) = &state.room_id {
                    let room_tag = create_room_tag(&config.game_id, room_id);
                    let content = serde_json::to_string(&EventContent::Heartbeat(HeartbeatEventContent {
                        timestamp: now_ms(),
                    }))
                    .unwrap();

                    if let Err(e) = client.publish_ephemeral(&room_tag, &content).await {
                        warn!("Failed to send heartbeat: {}", e);
                    }
                } else {
                    break;
                }
            }
        });
    }

    async fn start_presence_update(&self) {
        let client = self.client.clone();
        let room_state = self.room_state.clone();
        let players = self.players.clone();
        let config = self.config.clone();
        let event_tx = self.event_tx.clone();

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(30));

            loop {
                ticker.tick().await;

                let state = room_state.read().await;
                if state.room_id.is_none() || !state.is_host {
                    continue;
                }

                let room_id = state.room_id.clone().unwrap();
                let room_tag = create_room_tag(&config.game_id, &room_id);

                // Check for disconnected players
                let now = now_ms();
                let mut to_remove = Vec::new();

                {
                    let players_read = players.read().await;
                    for (pubkey, presence) in players_read.iter() {
                        if now - presence.last_seen > config.disconnect_threshold {
                            to_remove.push(pubkey.clone());
                        }
                    }
                }

                // Remove disconnected players
                for pubkey in to_remove {
                    players.write().await.remove(&pubkey);
                    let _ = event_tx.send(ArenaEvent::PlayerLeave(pubkey)).await;
                }

                // Publish updated room state
                let content = RoomEventContent {
                    status: state.status,
                    seed: state.seed,
                    host_pubkey: client.public_key(),
                    max_players: config.max_players,
                    expires_at: state.expires_at,
                    players: players.read().await.values().cloned().collect(),
                };

                if let Ok(json) = serde_json::to_string(&content) {
                    let _ = client.publish_room(&room_tag, &config.game_id, &json).await;
                }
            }
        });
    }

    async fn check_auto_start(&self) {
        if self.config.start_mode != StartMode::Auto {
            return;
        }

        let player_count = self.players.read().await.len();
        if player_count >= self.config.max_players {
            let mut state = self.room_state.write().await;
            state.status = RoomStatus::Playing;
            let _ = self.event_tx.send(ArenaEvent::GameStart).await;
        }
    }

    async fn check_all_ready(&self) {
        let all_ready = self.players.read().await.values().all(|p| p.ready);
        if !all_ready {
            return;
        }

        let _ = self.event_tx.send(ArenaEvent::AllReady).await;

        match self.config.start_mode {
            StartMode::Ready => {
                self.room_state.write().await.status = RoomStatus::Playing;
                let _ = self.event_tx.send(ArenaEvent::GameStart).await;
            }
            StartMode::Countdown => {
                let secs = self.config.countdown_seconds;
                let _ = self.event_tx.send(ArenaEvent::CountdownStart(secs)).await;

                // Simple countdown
                let event_tx = self.event_tx.clone();
                let room_state = self.room_state.clone();

                tokio::spawn(async move {
                    for i in (1..=secs).rev() {
                        let _ = event_tx.send(ArenaEvent::CountdownTick(i)).await;
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                    room_state.write().await.status = RoomStatus::Playing;
                    let _ = event_tx.send(ArenaEvent::GameStart).await;
                });
            }
            _ => {}
        }
    }

    async fn reset_for_rematch(&self, new_seed: u64) {
        let mut state = self.room_state.write().await;
        state.seed = new_seed;
        state.status = RoomStatus::Ready;
        state.rematch_requested = false;

        // Reset player ready states
        let mut players = self.players.write().await;
        for p in players.values_mut() {
            p.ready = false;
        }

        // Clear game states
        self.player_states.write().await.clear();

        let _ = self.event_tx.send(ArenaEvent::RematchStart(new_seed)).await;
    }
}

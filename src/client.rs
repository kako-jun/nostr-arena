//! Nostr client wrapper

use crate::error::{ArenaError, Result};
use crate::types::kinds;
use nostr_sdk::prelude::*;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Nostr client for arena operations
pub struct NostrClient {
    client: Client,
    relays: Vec<String>,
    connected: Arc<RwLock<bool>>,
    public_key: String,
}

impl NostrClient {
    /// Create a new NostrClient with generated keys
    pub async fn new(relays: Vec<String>) -> Result<Self> {
        let keys = Keys::generate();
        let public_key = keys.public_key().to_hex();
        let client = Client::new(keys);

        Ok(Self {
            client,
            relays,
            connected: Arc::new(RwLock::new(false)),
            public_key,
        })
    }

    /// Create a new NostrClient with provided secret key
    pub async fn with_secret_key(secret_key: &str, relays: Vec<String>) -> Result<Self> {
        let keys = Keys::parse(secret_key).map_err(|e| ArenaError::Nostr(e.to_string()))?;
        let public_key = keys.public_key().to_hex();
        let client = Client::new(keys);

        Ok(Self {
            client,
            relays,
            connected: Arc::new(RwLock::new(false)),
            public_key,
        })
    }

    /// Get the public key
    pub fn public_key(&self) -> String {
        self.public_key.clone()
    }

    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }

    /// Connect to relays
    pub async fn connect(&self) -> Result<()> {
        for relay in &self.relays {
            if let Err(e) = self.client.add_relay(relay).await {
                warn!("Failed to add relay {}: {}", relay, e);
            }
        }

        self.client.connect().await;
        *self.connected.write().await = true;
        debug!("Connected to relays");
        Ok(())
    }

    /// Disconnect from relays
    pub async fn disconnect(&self) -> Result<()> {
        let _ = self.client.disconnect().await;
        *self.connected.write().await = false;
        debug!("Disconnected from relays");
        Ok(())
    }

    /// Get relay connection status
    pub async fn relay_status(&self) -> Vec<(String, bool)> {
        let mut status = Vec::new();
        for relay in self.client.relays().await.values() {
            status.push((relay.url().to_string(), relay.is_connected()));
        }
        status
    }

    /// Check if at least one relay is connected
    pub async fn has_connected_relay(&self) -> bool {
        for relay in self.client.relays().await.values() {
            if relay.is_connected() {
                return true;
            }
        }
        false
    }

    /// Publish a room event (kind 30078)
    pub async fn publish_room(
        &self,
        d_tag: &str,
        game_id: &str,
        content: &str,
    ) -> Result<EventId> {
        let builder = EventBuilder::new(Kind::Custom(kinds::ROOM), content)
            .tags(vec![
                Tag::identifier(d_tag),
                Tag::hashtag(game_id),
            ]);

        let output = self
            .client
            .send_event_builder(builder)
            .await
            .map_err(|e| ArenaError::Nostr(e.to_string()))?;

        debug!("Published room event: {}", output.id());
        Ok(*output.id())
    }

    /// Publish an ephemeral event (kind 25000)
    pub async fn publish_ephemeral(&self, d_tag: &str, content: &str) -> Result<EventId> {
        let builder = EventBuilder::new(Kind::Custom(kinds::EPHEMERAL), content)
            .tags(vec![Tag::identifier(d_tag)]);

        let output = self
            .client
            .send_event_builder(builder)
            .await
            .map_err(|e| ArenaError::Nostr(e.to_string()))?;

        debug!("Published ephemeral event");
        Ok(*output.id())
    }

    /// Fetch room events
    pub async fn fetch_rooms(
        &self,
        game_id: &str,
        limit: usize,
    ) -> Result<Vec<Event>> {
        let filter = Filter::new()
            .kind(Kind::Custom(kinds::ROOM))
            .hashtag(game_id)
            .limit(limit);

        let events = self
            .client
            .fetch_events(vec![filter], std::time::Duration::from_secs(5))
            .await
            .map_err(|e| ArenaError::Nostr(e.to_string()))?;

        Ok(events.into_iter().collect())
    }

    /// Fetch a specific room by room tag
    pub async fn fetch_room(&self, d_tag: &str) -> Result<Option<Event>> {
        let filter = Filter::new()
            .kind(Kind::Custom(kinds::ROOM))
            .identifier(d_tag)
            .limit(1);

        let events = self
            .client
            .fetch_events(vec![filter], std::time::Duration::from_secs(5))
            .await
            .map_err(|e| ArenaError::Nostr(e.to_string()))?;

        Ok(events.into_iter().next())
    }

    /// Subscribe to room events
    pub async fn subscribe_room<F>(
        &self,
        d_tag: &str,
        callback: F,
    ) -> Result<SubscriptionId>
    where
        F: Fn(Event) + Send + Sync + 'static,
    {
        let filter = Filter::new()
            .kind(Kind::Custom(kinds::EPHEMERAL))
            .identifier(d_tag);

        let output = self
            .client
            .subscribe(vec![filter], None)
            .await
            .map_err(|e| ArenaError::Nostr(e.to_string()))?;

        let sub_id = output.id().clone();

        // Handle events in background
        let client = self.client.clone();
        let callback = Arc::new(callback);

        tokio::spawn(async move {
            let mut notifications = client.notifications();
            while let Ok(notification) = notifications.recv().await {
                if let RelayPoolNotification::Event { event, .. } = notification {
                    callback(*event);
                }
            }
        });

        debug!("Subscribed to room: {}", d_tag);
        Ok(sub_id)
    }

    /// Unsubscribe from a subscription
    pub async fn unsubscribe(&self, sub_id: SubscriptionId) -> Result<()> {
        self.client
            .unsubscribe(sub_id)
            .await;
        Ok(())
    }
}

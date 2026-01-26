//! Unit tests for nostr-arena

#[cfg(test)]
mod tests {
    use crate::types::*;

    #[test]
    fn test_arena_config_defaults() {
        let config = ArenaConfig::new("test-game");
        assert_eq!(config.game_id, "test-game");
        assert_eq!(config.max_players, 2);
        assert_eq!(config.room_expiry, 0);
        assert_eq!(config.countdown_seconds, 3);
        assert!(matches!(config.start_mode, StartMode::Auto));
    }

    #[test]
    fn test_arena_config_builder() {
        let config = ArenaConfig::new("test-game")
            .max_players(4)
            .room_expiry(600000)
            .start_mode(StartMode::Ready)
            .countdown_seconds(5)
            .base_url("https://example.com");

        assert_eq!(config.max_players, 4);
        assert_eq!(config.room_expiry, 600000);
        assert!(matches!(config.start_mode, StartMode::Ready));
        assert_eq!(config.countdown_seconds, 5);
        assert_eq!(config.base_url, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_room_state_default() {
        let state = RoomState::default();
        assert!(state.room_id.is_none());
        assert!(matches!(state.status, RoomStatus::Idle));
        assert!(!state.is_host);
    }

    #[test]
    fn test_player_presence() {
        let presence = PlayerPresence {
            pubkey: "abc123".to_string(),
            joined_at: 1000,
            last_seen: 2000,
            ready: true,
        };
        assert_eq!(presence.pubkey, "abc123");
        assert!(presence.ready);
    }

    #[test]
    fn test_room_status_variants() {
        let statuses = vec![
            RoomStatus::Idle,
            RoomStatus::Creating,
            RoomStatus::Waiting,
            RoomStatus::Joining,
            RoomStatus::Ready,
            RoomStatus::Playing,
            RoomStatus::Finished,
            RoomStatus::Deleted,
        ];
        assert_eq!(statuses.len(), 8);
    }

    #[test]
    fn test_start_mode_variants() {
        let modes = vec![
            StartMode::Auto,
            StartMode::Ready,
            StartMode::Countdown,
            StartMode::Host,
        ];
        assert_eq!(modes.len(), 4);
    }

    #[test]
    fn test_event_content_serialization() {
        use serde_json;

        // Test join event
        let join = EventContent::Join(JoinEventContent {
            player_pubkey: "abc123".to_string(),
        });
        let json = serde_json::to_string(&join).unwrap();
        assert!(json.contains("join"));
        assert!(json.contains("abc123"));

        // Test state event
        let state = EventContent::State(StateEventContent {
            game_state: serde_json::json!({"score": 100}),
        });
        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("game_state"));
        assert!(json.contains("100"));

        // Test ready event
        let ready = EventContent::Ready(ReadyEventContent { ready: true });
        let json = serde_json::to_string(&ready).unwrap();
        assert!(json.contains("ready"));
        assert!(json.contains("true"));

        // Test heartbeat event
        let heartbeat = EventContent::Heartbeat(HeartbeatEventContent { timestamp: 12345 });
        let json = serde_json::to_string(&heartbeat).unwrap();
        assert!(json.contains("heartbeat"));
        assert!(json.contains("12345"));

        // Test game over event
        let game_over = EventContent::GameOver(GameOverEventContent {
            reason: "win".to_string(),
            final_score: Some(100),
            winner: None,
        });
        let json = serde_json::to_string(&game_over).unwrap();
        assert!(json.contains("gameover"));
        assert!(json.contains("win"));
    }

    #[test]
    fn test_room_info() {
        let info = RoomInfo {
            room_id: "room123".to_string(),
            game_id: "test-game".to_string(),
            status: RoomStatus::Waiting,
            host_pubkey: "host123".to_string(),
            player_count: 1,
            max_players: 4,
            created_at: 1000,
            expires_at: Some(2000),
            seed: 12345,
        };
        assert_eq!(info.room_id, "room123");
        assert_eq!(info.player_count, 1);
        assert_eq!(info.max_players, 4);
    }
}

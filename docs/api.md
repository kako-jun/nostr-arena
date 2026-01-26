# nostr-arena API Reference

## ArenaConfig

Configuration for creating an Arena instance.

### Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `game_id` | string | required | Unique identifier for your game |
| `relays` | string[] | `["wss://relay.damus.io", ...]` | Nostr relay URLs |
| `room_expiry` | u64 | `0` (never) | Room expiration in ms |
| `max_players` | usize | `2` | Maximum players per room |
| `start_mode` | StartMode | `auto` | Game start mode |
| `countdown_seconds` | u32 | `3` | Countdown duration |
| `heartbeat_interval` | u64 | `3000` | Heartbeat interval in ms |
| `disconnect_threshold` | u64 | `10000` | Disconnect timeout in ms |
| `state_throttle` | u64 | `100` | State update throttle in ms |
| `base_url` | string? | none | Base URL for room URLs |

### Example

```rust
let config = ArenaConfig::new("my-game")
    .relays(vec!["wss://relay.damus.io".to_string()])
    .max_players(4)
    .start_mode(StartMode::Ready)
    .room_expiry(600000)  // 10 minutes
    .countdown_seconds(5)
    .base_url("https://example.com");
```

## Arena

Main class for managing multiplayer game rooms.

### Methods

#### Creating/Joining Rooms

| Method | Description |
|--------|-------------|
| `new(config)` | Create a new Arena instance |
| `connect()` | Connect to Nostr relays |
| `disconnect()` | Disconnect from relays |
| `create()` | Create a new room, returns URL |
| `join(room_id)` | Join an existing room |
| `leave()` | Leave the current room |
| `delete_room()` | Delete the room (host only) |

#### Game State

| Method | Description |
|--------|-------------|
| `send_state(state)` | Send game state to other players (throttled) |
| `send_game_over(reason, score?)` | Send game over event |
| `request_rematch()` | Request a rematch |
| `accept_rematch()` | Accept a rematch request |

#### Start Mode

| Method | Description |
|--------|-------------|
| `send_ready(ready)` | Send ready signal (Ready/Countdown modes) |
| `start_game()` | Start the game (Host mode, host only) |

#### QR Code

| Method | Description |
|--------|-------------|
| `get_room_url()` | Get the room URL |
| `get_room_qr_svg(options?)` | Get room QR code as SVG |
| `get_room_qr_data_url(options?)` | Get room QR code as data URL |

#### State

| Method | Description |
|--------|-------------|
| `public_key()` | Get this player's public key |
| `room_state()` | Get current room state |
| `players()` | Get list of players |
| `player_count()` | Get player count |
| `is_connected()` | Check if connected to relays |

#### Events

| Method | Description |
|--------|-------------|
| `recv()` | Wait for next event (blocking) |
| `try_recv()` | Poll for next event (non-blocking) |

#### Static Methods

| Method | Description |
|--------|-------------|
| `list_rooms(game_id, relays, status?, limit)` | List available rooms |

## ArenaEvent

Events emitted by the Arena.

| Event | Fields | Description |
|-------|--------|-------------|
| `PlayerJoin` | `player: PlayerPresence` | Player joined the room |
| `PlayerLeave` | `pubkey: String` | Player left the room |
| `PlayerState` | `pubkey, state` | Player's game state updated |
| `PlayerDisconnect` | `pubkey: String` | Player disconnected (heartbeat timeout) |
| `PlayerGameOver` | `pubkey, reason, final_score?` | Player sent game over |
| `RematchRequested` | `pubkey: String` | Player requested rematch |
| `RematchStart` | `seed: u64` | Rematch accepted, new seed provided |
| `AllReady` | - | All players are ready |
| `CountdownStart` | `seconds: u32` | Countdown started |
| `CountdownTick` | `remaining: u32` | Countdown tick |
| `GameStart` | - | Game started |
| `Error` | `message: String` | Error occurred |

## StartMode

| Mode | Description |
|------|-------------|
| `Auto` | Game starts immediately when max players join |
| `Ready` | Game starts when all players send ready signal |
| `Countdown` | Countdown starts when all players ready |
| `Host` | Host manually starts the game |

## RoomStatus

| Status | Description |
|--------|-------------|
| `Idle` | Not in a room |
| `Creating` | Creating a room |
| `Waiting` | Waiting for players |
| `Joining` | Joining a room |
| `Ready` | Room is ready, waiting to start |
| `Playing` | Game in progress |
| `Finished` | Game finished |
| `Deleted` | Room deleted |

## RoomInfo

Information about a room (from `list_rooms`).

| Field | Type | Description |
|-------|------|-------------|
| `room_id` | String | Room identifier |
| `game_id` | String | Game identifier |
| `status` | RoomStatus | Room status |
| `host_pubkey` | String | Host's public key |
| `player_count` | usize | Current player count |
| `max_players` | usize | Maximum players |
| `created_at` | u64 | Creation timestamp (ms) |
| `expires_at` | u64? | Expiration timestamp (ms) |
| `seed` | u64 | Random seed |

## PlayerPresence

Information about a player in the room.

| Field | Type | Description |
|-------|------|-------------|
| `pubkey` | String | Player's public key |
| `joined_at` | u64 | Join timestamp (ms) |
| `last_seen` | u64 | Last heartbeat timestamp (ms) |
| `ready` | bool | Ready status |

## QrOptions

Options for QR code generation.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `size` | u32? | 200 | Image size |
| `margin` | u32? | 4 | Quiet zone margin |
| `fg_color` | String? | "#000000" | Foreground color |
| `bg_color` | String? | "#ffffff" | Background color |
| `error_correction` | String? | - | Error correction level |

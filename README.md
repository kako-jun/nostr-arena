# nostr-arena

Nostr-based real-time multiplayer game arena. No server required.

Build multiplayer games that run over the decentralized Nostr network. Works with Rust, Python, and JavaScript/TypeScript.

## Features

- **Room Discovery** - Find available game rooms across relays
- **Multi-player** - Support for 2+ players per room
- **Presence Tracking** - Real-time player join/leave/heartbeat
- **Start Modes** - Auto, Ready, Countdown, or Host-controlled game start
- **QR Code** - Generate QR codes for easy room sharing
- **Cross-platform** - Rust core with bindings for npm (WASM) and PyPI

## Installation

### Rust

```toml
[dependencies]
nostr-arena-core = "0.2"
```

### Python

```bash
pip install nostr-arena
```

### JavaScript/TypeScript (npm)

```bash
npm install nostr-arena
```

## Quick Start

### Rust

```rust
use nostr_arena_core::{Arena, ArenaConfig, ArenaEvent, StartMode};
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
struct GameState {
    score: i32,
    position: (f32, f32),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ArenaConfig::new("my-game")
        .max_players(4)
        .start_mode(StartMode::Ready);

    let arena: Arena<GameState> = Arena::new(config).await?;
    arena.connect().await?;

    // Create a room
    let url = arena.create().await?;
    println!("Share this URL: {}", url);

    // Wait for events
    while let Some(event) = arena.recv().await {
        match event {
            ArenaEvent::PlayerJoin(player) => {
                println!("Player joined: {}", player.pubkey);
            }
            ArenaEvent::GameStart => {
                println!("Game started!");
            }
            ArenaEvent::PlayerState { pubkey, state } => {
                println!("Player {} score: {}", pubkey, state.score);
            }
            _ => {}
        }
    }

    Ok(())
}
```

### Python

```python
from nostr_arena import Arena, ArenaConfig
import json

config = (
    ArenaConfig("my-game")
    .max_players(4)
    .start_mode("ready")
)

arena = Arena(config)
arena.connect()

# Create a room
url = arena.create()
print(f"Share this URL: {url}")

# Game loop
while True:
    event = arena.try_recv()
    if event:
        if event.event_type == "player_join":
            print(f"Player joined: {event.player.pubkey}")
        elif event.event_type == "game_start":
            print("Game started!")
        elif event.event_type == "player_state":
            state = json.loads(event.state)
            print(f"Player {event.pubkey} score: {state['score']}")

    # Send your state
    arena.send_state(json.dumps({"score": 100, "x": 50, "y": 50}))
```

### JavaScript/TypeScript

```typescript
import { Arena, ArenaConfig } from 'nostr-arena';

interface GameState {
    score: number;
    position: { x: number; y: number };
}

const config = new ArenaConfig('my-game')
    .setMaxPlayers(4)
    .setStartMode('ready');

const arena = await Arena.init(config);
await arena.connect();

// Create a room
const url = await arena.create();
console.log('Share this URL:', url);

// Poll for events
setInterval(async () => {
    const event = await arena.tryRecv();
    if (event) {
        switch (event.type) {
            case 'playerJoin':
                console.log('Player joined:', event.player.pubkey);
                break;
            case 'gameStart':
                console.log('Game started!');
                break;
            case 'playerState':
                console.log(`Player ${event.pubkey} score:`, event.state.score);
                break;
        }
    }
}, 100);

// Send your state
await arena.sendState({ score: 100, position: { x: 50, y: 50 } });
```

## Room Discovery

Find available rooms:

```rust
// Rust
let rooms = Arena::<GameState>::list_rooms(
    "my-game",
    vec!["wss://relay.damus.io".to_string()],
    Some(RoomStatus::Waiting),
    10,
).await?;

for room in rooms {
    println!("{} - {} players", room.room_id, room.player_count);
}
```

```python
# Python
rooms = Arena.list_rooms(
    "my-game",
    ["wss://relay.damus.io"],
    "waiting",
    10
)

for room in rooms:
    print(f"{room.room_id} - {room.player_count} players")
```

```typescript
// JavaScript
const rooms = await Arena.listRooms(
    'my-game',
    ['wss://relay.damus.io'],
    'waiting',
    10
);

for (const room of rooms) {
    console.log(`${room.roomId} - ${room.playerCount} players`);
}
```

## Start Modes

| Mode | Description |
|------|-------------|
| `auto` | Game starts immediately when max players join |
| `ready` | Game starts when all players send ready signal |
| `countdown` | Countdown starts when all players ready |
| `host` | Host manually starts the game |

## QR Code

Generate QR codes for easy room sharing:

```rust
// Rust
let svg = arena.get_room_qr_svg(None).await;
let data_url = arena.get_room_qr_data_url(None).await;
```

```python
# Python
svg = arena.get_room_qr_svg()
data_url = arena.get_room_qr_data_url()
```

```typescript
// JavaScript
const svg = await arena.getRoomQRSvg();
const dataUrl = await arena.getRoomQRDataUrl();
```

## Configuration

| Option | Default | Description |
|--------|---------|-------------|
| `game_id` | required | Unique identifier for your game |
| `relays` | `["wss://relay.damus.io", ...]` | Nostr relay URLs |
| `room_expiry` | `0` (never) | Room expiration in ms |
| `max_players` | `2` | Maximum players per room |
| `start_mode` | `auto` | Game start mode |
| `countdown_seconds` | `3` | Countdown duration |
| `heartbeat_interval` | `3000` | Heartbeat interval in ms |
| `disconnect_threshold` | `10000` | Disconnect timeout in ms |
| `state_throttle` | `100` | State update throttle in ms |
| `base_url` | none | Base URL for room URLs |

## API

| Method | Description |
|--------|-------------|
| `connect()` | Connect to relays |
| `disconnect()` | Disconnect from relays |
| `create()` | Create a room, returns URL |
| `join(room_id)` | Join a room |
| `leave()` | Leave current room |
| `reconnect(room_id)` | Reconnect to a room (e.g., after page refresh) |
| `delete_room()` | Delete room (host only) |
| `send_state(state)` | Send game state |
| `send_game_over(reason, score)` | Send game over |
| `send_ready(ready)` | Send ready signal |
| `start_game()` | Start game (host only) |
| `request_rematch()` | Request rematch |
| `accept_rematch()` | Accept rematch |
| `try_recv()` | Poll for event (non-blocking) |
| `recv()` | Wait for event (blocking, Rust only) |
| `players()` | Get current players |
| `player_count()` | Get player count |
| `get_room_url()` | Get room URL |
| `get_room_qr_svg()` | Get QR code as SVG |
| `get_room_qr_data_url()` | Get QR code as data URL |
| `list_rooms()` | List available rooms (static) |

## Events

| Event | Description |
|-------|-------------|
| `PlayerJoin` | Player joined the room |
| `PlayerLeave` | Player left the room |
| `PlayerState` | Player's game state updated |
| `PlayerDisconnect` | Player disconnected (heartbeat timeout) |
| `PlayerGameOver` | Player sent game over |
| `RematchRequested` | Player requested rematch |
| `RematchStart` | Rematch accepted, new seed provided |
| `AllReady` | All players are ready |
| `CountdownStart` | Countdown started |
| `CountdownTick` | Countdown tick |
| `GameStart` | Game started |
| `Error` | Error occurred |

## Related Packages

- **Rust crate**: [nostr-arena](https://crates.io/crates/nostr-arena)
- **npm (WASM)**: [nostr-arena-js](https://github.com/kako-jun/nostr-arena-js)
- **PyPI**: [nostr-arena-python](https://github.com/kako-jun/nostr-arena-python)

## Building

```bash
cargo build --release
cargo test
```

## License

MIT

# Changelog

All notable changes to this project will be documented in this file.

## [0.2.0] - 2025-01-26

### Added
- **Room Discovery**: `list_rooms()` static method to find available game rooms
- **Multi-player Support**: Track multiple players per room (configurable via `max_players`)
- **Start Modes**: Four game start modes
  - `Auto`: Start when max players join
  - `Ready`: Start when all players send ready signal
  - `Countdown`: Countdown after all players ready
  - `Host`: Host manually starts the game
- **QR Code Generation**: `get_room_qr_svg()` and `get_room_qr_data_url()` methods
- **Presence Tracking**: Heartbeat-based player presence with disconnect detection
- **Reconnection**: `reconnect()` method for session recovery
- **Room Expiry**: Optional room expiration time
- **Room Deletion**: `delete_room()` for hosts to clean up rooms

### Changed
- Complete Rust rewrite (formerly TypeScript)
- Event-based API using async channels instead of callbacks
- Separate repositories for core (Rust), npm (WASM), and PyPI (Python)

### Events
- `PlayerJoin` - Player joined the room
- `PlayerLeave` - Player left the room
- `PlayerState` - Player's game state updated
- `PlayerDisconnect` - Player disconnected (heartbeat timeout)
- `PlayerGameOver` - Player sent game over
- `RematchRequested` - Player requested rematch
- `RematchStart` - Rematch accepted, new game starting
- `AllReady` - All players are ready
- `CountdownStart` - Countdown started
- `CountdownTick` - Countdown tick
- `GameStart` - Game started
- `Error` - Error occurred

## [0.1.0] - 2025-01-05

### Added
- Initial TypeScript implementation
- Basic room creation and joining
- 1v1 game state synchronization
- Rematch functionality

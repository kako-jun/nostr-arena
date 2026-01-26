# nostr-arena Architecture

## Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     nostr-arena-core (Rust)                 │
│                                                             │
│  ┌─────────┐  ┌─────────────┐  ┌──────────┐  ┌──────────┐  │
│  │  Arena  │──│ NostrClient │──│  Types   │──│  Error   │  │
│  └─────────┘  └─────────────┘  └──────────┘  └──────────┘  │
│       │              │                                      │
│       └──────────────┼──────────────────────────────────────┤
│                      │                                      │
│              ┌───────┴───────┐                              │
│              │   nostr-sdk   │                              │
│              └───────────────┘                              │
└─────────────────────────────────────────────────────────────┘
                           │
         ┌─────────────────┼─────────────────┐
         │                 │                 │
         ▼                 ▼                 ▼
┌─────────────────┐ ┌─────────────┐ ┌─────────────────┐
│  bindings/wasm  │ │bindings/py  │ │  Native Rust    │
│  (wasm-pack)    │ │ (maturin)   │ │     Apps        │
└─────────────────┘ └─────────────┘ └─────────────────┘
         │                 │                 │
         ▼                 ▼                 ▼
┌─────────────────┐ ┌─────────────┐ ┌─────────────────┐
│   npm package   │ │ PyPI package│ │  Rust TUI/CLI   │
│  (JavaScript)   │ │  (Python)   │ │     Games       │
└─────────────────┘ └─────────────┘ └─────────────────┘
```

## Components

### nostr-arena-core

The core library written in Rust. Provides:

- **Arena**: Main game room management
- **NostrClient**: Nostr protocol handling via nostr-sdk
- **Types**: Shared type definitions
- **Error**: Error types

### bindings/wasm

WebAssembly bindings built with wasm-bindgen. Exports:

- `Arena` class
- `ArenaConfig` class
- Event types

Built with `wasm-pack` for npm distribution.

### bindings/python

Python bindings built with PyO3/maturin. Exports:

- `Arena` class
- `ArenaConfig` class
- `ArenaEvent`, `PlayerPresence`, `RoomInfo` classes

Built with `maturin` for PyPI distribution.

## Data Flow

### State Machine

```
┌──────┐  create()  ┌──────────┐  join()  ┌─────────┐
│ Idle │──────────► │ Creating │────────► │ Waiting │
└──────┘            └──────────┘          └────┬────┘
    ▲                                          │
    │                              opponent joins / auto
    │                                          │
    │               ┌──────────┐               ▼
    │  leave()      │ Finished │◄────────┌─────────┐
    └───────────────┤          │  game   │ Playing │
                    └────┬─────┘  over   └─────────┘
                         │                    ▲
                         │ rematch            │ ready / start
                         │                    │
                         └────────────────────┘
```

### Event Flow

```
┌─────────────────────────────────────────────────────────────┐
│                        Application                          │
│                                                             │
│   ┌─────────────┐          ┌─────────────┐                  │
│   │ Game Logic  │◄────────►│   Arena     │                  │
│   └─────────────┘ events   └──────┬──────┘                  │
│                                   │                          │
└───────────────────────────────────┼──────────────────────────┘
                                    │
                    ┌───────────────┼───────────────┐
                    │               ▼               │
                    │   ┌───────────────────┐       │
                    │   │   NostrClient     │       │
                    │   └─────────┬─────────┘       │
                    │             │                 │
                    │   ┌─────────┴─────────┐       │
                    │   │    nostr-sdk      │       │
                    │   └─────────┬─────────┘       │
                    │             │                 │
                    └─────────────┼─────────────────┘
                                  │
                    ┌─────────────┴─────────────┐
                    │                           │
                    ▼                           ▼
            ┌─────────────┐             ┌─────────────┐
            │   Relay 1   │             │   Relay 2   │
            └─────────────┘             └─────────────┘
```

## Multi-player State

```
┌─────────────────────────────────────────────────────────────┐
│                         Arena                               │
│                                                             │
│   ┌───────────────────────────────────────────────────┐     │
│   │                    RoomState                      │     │
│   │   room_id, status, is_host, seed, expires_at      │     │
│   └───────────────────────────────────────────────────┘     │
│                                                             │
│   ┌───────────────────────────────────────────────────┐     │
│   │              players: HashMap<pubkey, ...>        │     │
│   │                                                   │     │
│   │   ┌─────────────┐  ┌─────────────┐               │     │
│   │   │ Player A    │  │ Player B    │  ...          │     │
│   │   │ pubkey      │  │ pubkey      │               │     │
│   │   │ joined_at   │  │ joined_at   │               │     │
│   │   │ last_seen   │  │ last_seen   │               │     │
│   │   │ ready       │  │ ready       │               │     │
│   │   └─────────────┘  └─────────────┘               │     │
│   └───────────────────────────────────────────────────┘     │
│                                                             │
│   ┌───────────────────────────────────────────────────┐     │
│   │          player_states: HashMap<pubkey, T>        │     │
│   │                                                   │     │
│   │   ┌─────────────┐  ┌─────────────┐               │     │
│   │   │ State A     │  │ State B     │  ...          │     │
│   │   │ (generic T) │  │ (generic T) │               │     │
│   │   └─────────────┘  └─────────────┘               │     │
│   └───────────────────────────────────────────────────┘     │
│                                                             │
│   ┌───────────────────────────────────────────────────┐     │
│   │                event_tx/event_rx                  │     │
│   │           (mpsc channel for events)               │     │
│   └───────────────────────────────────────────────────┘     │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Start Mode Flow

### Auto Mode

```
Player 1 joins  ─────►  Player 2 joins  ─────►  Game starts
                        (max_players reached)
```

### Ready Mode

```
Player 1 joins  ─────►  Player 2 joins
      │                       │
      ▼                       ▼
Player 1 ready  ─────►  Player 2 ready  ─────►  Game starts
```

### Countdown Mode

```
Player 1 ready  ─────►  Player 2 ready  ─────►  Countdown
                                                    │
                                              3... 2... 1...
                                                    │
                                                    ▼
                                              Game starts
```

### Host Mode

```
Player 1 joins  ─────►  Player 2 joins
      │
      │ (Host)
      ▼
Host clicks start  ─────────────────────────►  Game starts
```

## Presence Tracking

```
┌─────────────────────────────────────────────────────────────┐
│                     Heartbeat Loop                          │
│                                                             │
│   Every 3 seconds:                                          │
│     - Publish heartbeat event to room                       │
│                                                             │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                  Presence Update Loop (Host)                │
│                                                             │
│   Every 30 seconds:                                         │
│     - Check each player's last_seen                         │
│     - Remove players with last_seen > 10 seconds ago        │
│     - Publish updated room event with player list           │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Directory Structure

```
nostr-arena/
├── Cargo.toml                 # Workspace config
├── README.md
├── crates/
│   └── nostr-arena-core/      # Rust core library
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── arena.rs       # Main Arena struct
│           ├── client.rs      # NostrClient wrapper
│           ├── types.rs       # Type definitions
│           └── error.rs       # Error types
├── bindings/
│   ├── wasm/                  # npm WASM bindings
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   └── python/                # PyPI bindings
│       ├── Cargo.toml
│       ├── pyproject.toml
│       └── src/lib.rs
├── examples/
│   ├── rust-tui/              # Rust TUI example
│   ├── pygame/                # Python game example
│   └── web/                   # Browser example
└── docs/
    ├── protocol.md            # Nostr protocol spec
    └── architecture.md        # This file
```

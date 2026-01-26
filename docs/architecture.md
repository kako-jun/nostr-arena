# nostr-arena Architecture

## Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     nostr-arena (Rust)                      │
│                                                             │
│  ┌─────────┐  ┌─────────────┐  ┌──────────┐  ┌──────────┐  │
│  │  Arena  │──│ NostrClient │──│  Types   │──│  Error   │  │
│  └─────────┘  └─────────────┘  └──────────┘  └──────────┘  │
│       │              │              │              │        │
│       └──────────────┼──────────────┼──────────────┘        │
│                      │              │                       │
│              ┌───────┴───────┐  ┌───┴───┐                   │
│              │   nostr-sdk   │  │qrcode │                   │
│              └───────────────┘  └───────┘                   │
└─────────────────────────────────────────────────────────────┘
                           │
         ┌─────────────────┼─────────────────┐
         │                 │                 │
         ▼                 ▼                 ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│ nostr-arena-js  │ │nostr-arena-py   │ │  Native Rust    │
│   (NAPI-RS)     │ │    (PyO3)       │ │     Apps        │
└─────────────────┘ └─────────────────┘ └─────────────────┘
         │                 │                 │
         ▼                 ▼                 ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│   npm package   │ │  PyPI package   │ │  Rust TUI/CLI   │
│  (JavaScript)   │ │    (Python)     │ │     Games       │
└─────────────────┘ └─────────────────┘ └─────────────────┘
```

## Repositories

| Repository | Description | Package |
|------------|-------------|---------|
| [nostr-arena](https://github.com/kako-jun/nostr-arena) | Rust core library | crates.io |
| [nostr-arena-js](https://github.com/kako-jun/nostr-arena-js) | WASM bindings | npm |
| [nostr-arena-python](https://github.com/kako-jun/nostr-arena-python) | Python bindings | PyPI |

## Components

### nostr-arena (Core)

The core library written in Rust. Provides:

- **Arena**: Main game room management
- **NostrClient**: Nostr protocol handling via nostr-sdk
- **Types**: Shared type definitions
- **Error**: Error types
- **QR**: QR code generation

### nostr-arena-js

Native Node.js bindings built with NAPI-RS. Exports:

- `Arena` class
- `listRooms()` function
- Event types

Built with `napi-rs` for npm distribution. Runs as native code (not WebAssembly).

### nostr-arena-python

Python bindings built with PyO3. Exports:

- `Arena` class
- `ArenaConfig` class
- Event dictionaries

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

### nostr-arena (Core)

```
nostr-arena/
├── Cargo.toml
├── README.md
├── LICENSE
├── .pre-commit-config.yaml
├── .github/
│   └── workflows/
│       ├── ci.yml
│       └── release.yml
├── src/
│   ├── lib.rs
│   ├── arena.rs      # Main Arena struct
│   ├── client.rs     # NostrClient wrapper
│   ├── types.rs      # Type definitions
│   ├── error.rs      # Error types
│   ├── qr.rs         # QR code generation
│   └── tests.rs      # Unit tests
├── examples/
│   └── tui.rs        # TUI example
└── docs/
    ├── protocol.md
    ├── api.md
    └── architecture.md
```

### nostr-arena-js (npm)

```
nostr-arena-js/
├── Cargo.toml
├── package.json
├── package-lock.json
├── build.rs
├── README.md
├── LICENSE
├── .pre-commit-config.yaml
├── .github/
│   └── workflows/
│       ├── ci.yml
│       └── release.yml
└── src/
    └── lib.rs        # NAPI-RS bindings
```

### nostr-arena-python (PyPI)

```
nostr-arena-python/
├── Cargo.toml
├── pyproject.toml
├── README.md
├── LICENSE
├── .pre-commit-config.yaml
├── .github/
│   └── workflows/
│       ├── ci.yml
│       └── release.yml
├── src/
│   └── lib.rs        # Python bindings
└── python/
    └── nostr_arena/
        └── __init__.py
```

## Build Limitations

### Supported Platforms

| Platform | nostr-arena-js | nostr-arena-python |
|----------|----------------|-------------------|
| Linux x86_64 | ✅ | ✅ |
| Linux ARM64 | ❌ | ❌ |
| macOS x86_64 | ✅ | ✅ |
| macOS ARM64 | ✅ | ✅ |
| Windows x86_64 | ✅ | ✅ |

### ARM Linux Cross-Compilation Issue

The `ring` crate (used by `nostr-sdk` for TLS/cryptography) cannot be cross-compiled for ARM Linux from x86 machines. This is because `ring` uses hand-written assembly code that requires proper ARM toolchain configuration.

**Error message:**
```
#error "ARM assembler must define __ARM_ARCH"
```

**Why it happens:**
- `ring` contains optimized assembly for cryptographic operations
- Cross-compiling from x86 to ARM requires the ARM assembler to define `__ARM_ARCH`
- GitHub Actions x86 runners don't have properly configured ARM cross-compilation toolchains for `ring`

**Why macOS ARM works:**
- `macos-latest` GitHub runner is native ARM hardware
- No cross-compilation needed; builds natively

**Workarounds:**
1. Use native ARM runners (GitHub Actions ARM runners are available but limited)
2. Wait for `nostr-sdk` to support alternative TLS backends (e.g., `rustls` without `ring`)
3. Build on actual ARM hardware

**Dependency chain:**
```
nostr-arena
    └── nostr-sdk
            └── reqwest (TLS)
                    └── ring  ← Problem here
```

# nostr-arena Protocol

nostr-arena uses Nostr events to coordinate multiplayer games without a central server.

## Event Kinds

| Kind | Type | Description |
|------|------|-------------|
| 30078 | Replaceable | Room metadata (NIP-78) |
| 25000 | Ephemeral | Game events (not stored) |

## Room Event (kind 30078)

Room events are replaceable events that store room metadata.

### Tags

- `d`: Room identifier (format: `{gameId}-{roomId}`)
- `t`: Game ID hashtag for discovery

### Content

```json
{
    "status": "waiting",
    "seed": 1234567890,
    "host_pubkey": "abc123...",
    "max_players": 4,
    "expires_at": 1704000000000,
    "players": [
        {
            "pubkey": "abc123...",
            "joined_at": 1704000000000,
            "last_seen": 1704000100000,
            "ready": true
        }
    ]
}
```

### Status Values

- `waiting` - Waiting for players
- `playing` - Game in progress
- `finished` - Game completed
- `deleted` - Room deleted

## Ephemeral Events (kind 25000)

Ephemeral events are used for real-time game communication. They are not stored by relays.

### Tags

- `d`: Room identifier (format: `{gameId}-{roomId}`)

### Event Types

#### join

Sent when a player joins a room.

```json
{
    "type": "join",
    "player_pubkey": "abc123..."
}
```

#### state

Sent to sync game state.

```json
{
    "type": "state",
    "game_state": {
        "score": 100,
        "position": { "x": 50, "y": 50 }
    }
}
```

#### heartbeat

Sent periodically to indicate presence.

```json
{
    "type": "heartbeat",
    "timestamp": 1704000000000
}
```

#### ready

Sent to indicate ready status (for Ready/Countdown modes).

```json
{
    "type": "ready",
    "ready": true
}
```

#### gamestart

Sent by host to start the game (for Host mode).

```json
{
    "type": "gamestart"
}
```

#### gameover

Sent when a player's game ends.

```json
{
    "type": "gameover",
    "reason": "collision",
    "final_score": 1500,
    "winner": "def456..."
}
```

#### rematch

Sent to request or accept a rematch.

```json
{
    "type": "rematch",
    "action": "request"
}
```

```json
{
    "type": "rematch",
    "action": "accept",
    "new_seed": 9876543210
}
```

## Flow Diagrams

### Room Creation

```
Host                    Relay
  |                       |
  |-- publish kind 30078 -->|  (room, status=waiting)
  |                       |
  |<-- subscribe kind 25000 --|
  |                       |
```

### Player Join

```
Host                    Relay                   Guest
  |                       |                       |
  |                       |<-- fetch kind 30078 --|
  |                       |                       |
  |                       |-- room data --------->|
  |                       |                       |
  |                       |<-- publish kind 25000 (join) --|
  |                       |                       |
  |<-- event (join) ------|                       |
  |                       |                       |
```

### State Sync

```
Player A                Relay                   Player B
  |                       |                       |
  |-- publish kind 25000 -->|                       |
  |   (state)             |                       |
  |                       |-- event (state) ----->|
  |                       |                       |
  |                       |<-- publish kind 25000 --|
  |                       |   (state)             |
  |<-- event (state) -----|                       |
  |                       |                       |
```

### Ready Mode Start

```
Player A                Relay                   Player B
  |                       |                       |
  |-- publish (ready) --->|                       |
  |                       |-- event (ready) ----->|
  |                       |                       |
  |                       |<-- publish (ready) ---|
  |<-- event (ready) -----|                       |
  |                       |                       |
  |   [All ready - game starts locally]           |
  |                       |                       |
```

### Host Mode Start

```
Host                    Relay                   Guest
  |                       |                       |
  |-- publish (gamestart) -->|                    |
  |                       |-- event (gamestart) ->|
  |                       |                       |
  |   [Game starts]       |       [Game starts]   |
  |                       |                       |
```

## Presence Tracking

Players send heartbeat events every 3 seconds (configurable). If no heartbeat is received for 10 seconds (configurable), the player is considered disconnected.

The host updates the room event every 30 seconds with the current player list, removing players who have timed out.

## Room Expiration

If `expires_at` is set, the room is considered expired after that timestamp. Expired rooms:

- Are excluded from `list_rooms` results
- Cannot be joined
- May be overwritten by new rooms with the same ID

## Recommended Relays

- `wss://relay.damus.io`
- `wss://nos.lol`
- `wss://relay.nostr.band`
- `wss://relay.snort.social`

Using multiple relays improves reliability and reduces latency.

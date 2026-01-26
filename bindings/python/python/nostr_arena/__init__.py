"""
nostr-arena - Nostr-based real-time multiplayer game arena

No server required. Peer-to-peer game networking over Nostr relays.

Example:
    from nostr_arena import Arena, ArenaConfig

    config = ArenaConfig("my-game").max_players(4).start_mode("ready")
    arena = Arena(config)
    arena.connect()

    # Create a room
    url = arena.create()
    print(f"Share this URL: {url}")

    # Poll for events
    while True:
        event = arena.try_recv()
        if event:
            if event.event_type == "player_join":
                print(f"Player joined: {event.player.pubkey}")
            elif event.event_type == "game_start":
                print("Game started!")
"""

from nostr_arena._core import (
    Arena,
    ArenaConfig,
    ArenaEvent,
    PlayerPresence,
    RoomInfo,
)

__all__ = [
    "Arena",
    "ArenaConfig",
    "ArenaEvent",
    "PlayerPresence",
    "RoomInfo",
]

__version__ = "0.2.0"

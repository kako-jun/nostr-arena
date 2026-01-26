"""
PyGame example for nostr-arena

A simple multiplayer game where players move colored squares.

Requirements:
    pip install pygame nostr-arena
"""

import pygame
import json
import sys
from nostr_arena import Arena, ArenaConfig

# Constants
SCREEN_WIDTH = 800
SCREEN_HEIGHT = 600
PLAYER_SIZE = 30
MOVE_SPEED = 5

# Colors
WHITE = (255, 255, 255)
BLACK = (0, 0, 0)
COLORS = [
    (255, 0, 0),    # Red
    (0, 255, 0),    # Green
    (0, 0, 255),    # Blue
    (255, 255, 0),  # Yellow
]


class Player:
    def __init__(self, pubkey: str, color_index: int):
        self.pubkey = pubkey
        self.x = SCREEN_WIDTH // 2
        self.y = SCREEN_HEIGHT // 2
        self.color = COLORS[color_index % len(COLORS)]

    def draw(self, screen):
        pygame.draw.rect(
            screen,
            self.color,
            (self.x - PLAYER_SIZE // 2, self.y - PLAYER_SIZE // 2, PLAYER_SIZE, PLAYER_SIZE)
        )
        # Draw pubkey label
        font = pygame.font.Font(None, 20)
        label = font.render(self.pubkey[:8], True, BLACK)
        screen.blit(label, (self.x - 20, self.y - PLAYER_SIZE // 2 - 15))


class Game:
    def __init__(self):
        pygame.init()
        self.screen = pygame.display.set_mode((SCREEN_WIDTH, SCREEN_HEIGHT))
        pygame.display.set_caption("nostr-arena PyGame Example")
        self.clock = pygame.time.Clock()
        self.font = pygame.font.Font(None, 36)

        # Arena setup
        config = (
            ArenaConfig("pygame-example")
            .max_players(4)
            .start_mode("auto")
            .base_url("https://example.com")
        )
        self.arena = Arena(config)
        self.arena.connect()

        # Game state
        self.players = {}
        self.my_player = None
        self.room_url = None
        self.game_started = False
        self.status = "Press C to create room or J to join"

    def handle_events(self):
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                return False
            elif event.type == pygame.KEYDOWN:
                if event.key == pygame.K_c and not self.room_url:
                    self.create_room()
                elif event.key == pygame.K_j and not self.room_url:
                    self.join_room()
                elif event.key == pygame.K_ESCAPE:
                    return False
        return True

    def create_room(self):
        try:
            self.room_url = self.arena.create()
            self.status = f"Room created! URL: {self.room_url}"
            self.my_player = Player(self.arena.public_key(), 0)
            self.players[self.arena.public_key()] = self.my_player
            print(f"Room URL: {self.room_url}")
        except Exception as e:
            self.status = f"Failed to create room: {e}"

    def join_room(self):
        # For demo, prompt for room ID
        room_id = input("Enter room ID: ")
        try:
            self.arena.join(room_id)
            self.status = f"Joined room: {room_id}"
            self.my_player = Player(self.arena.public_key(), len(self.players))
            self.players[self.arena.public_key()] = self.my_player
        except Exception as e:
            self.status = f"Failed to join: {e}"

    def handle_arena_events(self):
        while True:
            event = self.arena.try_recv()
            if event is None:
                break

            if event.event_type == "player_join":
                pubkey = event.player.pubkey
                if pubkey not in self.players:
                    self.players[pubkey] = Player(pubkey, len(self.players))
                    self.status = f"Player joined: {pubkey[:8]}"

            elif event.event_type == "player_leave":
                if event.pubkey in self.players:
                    del self.players[event.pubkey]
                    self.status = f"Player left: {event.pubkey[:8]}"

            elif event.event_type == "player_state":
                if event.pubkey and event.state:
                    state = json.loads(event.state)
                    if event.pubkey in self.players:
                        self.players[event.pubkey].x = state.get("x", 0)
                        self.players[event.pubkey].y = state.get("y", 0)

            elif event.event_type == "game_start":
                self.game_started = True
                self.status = "Game started!"

    def update(self):
        if not self.my_player or not self.game_started:
            return

        # Handle movement
        keys = pygame.key.get_pressed()
        moved = False

        if keys[pygame.K_LEFT] or keys[pygame.K_a]:
            self.my_player.x = max(PLAYER_SIZE // 2, self.my_player.x - MOVE_SPEED)
            moved = True
        if keys[pygame.K_RIGHT] or keys[pygame.K_d]:
            self.my_player.x = min(SCREEN_WIDTH - PLAYER_SIZE // 2, self.my_player.x + MOVE_SPEED)
            moved = True
        if keys[pygame.K_UP] or keys[pygame.K_w]:
            self.my_player.y = max(PLAYER_SIZE // 2, self.my_player.y - MOVE_SPEED)
            moved = True
        if keys[pygame.K_DOWN] or keys[pygame.K_s]:
            self.my_player.y = min(SCREEN_HEIGHT - PLAYER_SIZE // 2, self.my_player.y + MOVE_SPEED)
            moved = True

        # Send state if moved
        if moved:
            state = json.dumps({"x": self.my_player.x, "y": self.my_player.y})
            try:
                self.arena.send_state(state)
            except Exception as e:
                print(f"Failed to send state: {e}")

    def draw(self):
        self.screen.fill(WHITE)

        # Draw players
        for player in self.players.values():
            player.draw(self.screen)

        # Draw status
        status_text = self.font.render(self.status, True, BLACK)
        self.screen.blit(status_text, (10, 10))

        # Draw player count
        count_text = self.font.render(f"Players: {len(self.players)}", True, BLACK)
        self.screen.blit(count_text, (10, 50))

        pygame.display.flip()

    def run(self):
        running = True
        while running:
            running = self.handle_events()
            self.handle_arena_events()
            self.update()
            self.draw()
            self.clock.tick(60)

        self.arena.disconnect()
        pygame.quit()


if __name__ == "__main__":
    game = Game()
    game.run()

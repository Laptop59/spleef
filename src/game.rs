use crate::arena::{ArenaError, Location, Region};
use crate::config::Configuration;
use crate::data::SpleefData;
use pumpkin_plugin_api::Server;
use pumpkin_plugin_api::common::NamedColor;
use pumpkin_plugin_api::text::TextComponent;
use std::cmp::PartialEq;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// A structure that manages all game instances.
#[derive(Default)]
pub struct GameManager {
    /// A map that keeps track of the active games.
    ///
    /// The key of the map is the same as the name
    /// of the arena it is associated with.
    games: HashMap<String, ActiveGame>,

    /// A map that keeps track of which games
    /// each opting player is in.
    players: HashMap<Uuid, String>,
}

impl GameManager {
    pub fn create_new(
        &mut self,
        configuration: &mut Configuration,
        arena: &str,
    ) -> Result<(), ArenaError> {
        let game = ActiveGame::try_new(configuration, arena)?;
        self.games.insert(arena.to_string(), game);
        Ok(())
    }

    pub fn has(&self, arena: &str) -> bool {
        self.games.contains_key(arena)
    }

    pub fn get(&self, arena: &str) -> Option<&ActiveGame> {
        self.games.get(arena)
    }

    pub fn get_mut(&mut self, arena: &str) -> Option<&mut ActiveGame> {
        self.games.get_mut(arena)
    }

    pub fn remove(&mut self, arena: &str) -> bool {
        self.games.remove(arena).is_some()
    }

    pub fn join_player(
        &mut self,
        arena: &str,
        player: Uuid,
        server: &Server,
    ) -> Result<(), ArenaError> {
        if self.get(arena).is_some_and(ActiveGame::is_full) {
            return Err(ArenaError::GameAlreadyStarted);
        }
        self.add_player(arena, player, server)
    }

    pub fn add_player(&mut self, arena: &str, player: Uuid, server: &Server) -> Result<(), ArenaError> {
        if let Some(current_game) = self.players.get(&player) {
            if current_game == arena {
                return Err(ArenaError::AlreadyJoinedGame);
            }
            // Remove the player from an existing game.
            self.remove_player(&player, server);
        }
        let mut added_player = false;
        if let Some(game) = self.get_mut(arena) {
            game.add_player(player);
            Self::broadcast_player_join_or_leave(
                game,
                &player,
                server,
                JoinOrLeave::Join,
            );
            added_player = true;
        }

        if added_player {
            self.players.insert(player, arena.to_string());
        }

        Ok(())
    }

    pub fn remove_player(&mut self, player: &Uuid, server: &Server) {
        if let Some(game) = self.players.get(player) {
            if let Some(game) = self.games.get_mut(game) {
                game.remove_player(player);
                Self::broadcast_player_join_or_leave(game, player, server, JoinOrLeave::Leave);
            }
        }
    }

    fn broadcast_player_join_or_leave(
        game: &ActiveGame,
        player: &Uuid,
        server: &Server,
        join_or_leave: JoinOrLeave,
    ) {
        if game.status != GameStatus::Queuing {
            return;
        }
        game.broadcast(server, || {
            let player_name = server
                .get_player_by_uuid(&player.to_string())
                .map(|p| p.get_display_name())
                .unwrap_or(TextComponent::text("???"));

            let color = join_or_leave.color();

            let text = TextComponent::text("");
            text.add_child({
                let text = TextComponent::text("[");
                text.color_named(NamedColor::DarkGray);
                text
            });
            text.add_child({
                let text = TextComponent::text(join_or_leave.symbol());
                text.color_named(color);
                text
            });
            text.add_child({
                let text = TextComponent::text("] ");
                text.color_named(NamedColor::DarkGray);
                text
            });
            text.add_child(player_name);
            text.add_child({
                let text = TextComponent::text(join_or_leave.message());
                text.color_named(color);
                text
            });

            if matches!(join_or_leave, JoinOrLeave::Join) {
                text.add_child({
                    let text = TextComponent::text(" (");
                    text.color_named(NamedColor::Gray);
                    text
                });
                text.add_child({
                    let text = TextComponent::text(&game.players.len().to_string());
                    text.color_named(NamedColor::White);
                    text
                });
                if let Some(max_players) = game.max_players {
                    text.add_child({
                        let text = TextComponent::text("/");
                        text.color_named(NamedColor::Gray);
                        text
                    });
                    text.add_child({
                        let text = TextComponent::text(&max_players.to_string());
                        text.color_named(NamedColor::White);
                        text
                    });
                }
                text.add_child({
                    let text = TextComponent::text(")");
                    text.color_named(NamedColor::Gray);
                    text
                });
            }
            text
        });
    }
}

/// Represents a currently active game.
pub struct ActiveGame {
    /// All the players who are currently in this game.
    players: HashSet<Uuid>,

    /// The name of the arena locked by this game.
    arena: String,

    /// The current status of this game.
    status: GameStatus,

    /// The region compromising the entire arena.
    pub map_region: Option<Region>,

    /// Represents an optional death zone.
    /// This is optional because blocks like lava
    /// kill the player anyway.
    pub death_zone: Option<Region>,

    /// Represents the spawn location of the players upon the game starting.
    pub spawn: Location,

    /// Represents the lobby location of the players before the game starts.
    /// If unspecified, this will default to the spawn point.
    pub lobby: Location,

    /// Represents the spectator spawning location of the players when they die.
    pub spectator: Location,

    /// Minimum number of players required to start the game.
    pub min_players: usize,

    /// Maximum number of players required to start the game.
    /// If [`None`], this is assumed to be infinity.
    pub max_players: Option<usize>,

    /// Materials allowed to be broken for the game.
    /// Strings here are blocks.
    ///
    /// By default, it is set to be snow blocks only.
    pub materials: Vec<String>,
}

#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum GameStatus {
    #[default]
    Queuing,
    Ongoing,
    Ended,
}

impl ActiveGame {
    fn try_new(
        configuration: &mut Configuration,
        arena_name: &str,
    ) -> Result<ActiveGame, ArenaError> {
        let arena = configuration.get_arena_mut(arena_name)?;
        if !arena.errors().is_empty() {
            return Err(ArenaError::UnresolvedArenaErrors);
        }
        // Create the new game since the arena is unoccupied
        // (The unoccupied check was already done in `get_arena_mut`)
        arena.occupied = true;

        let spawn = arena
            .spawn
            .expect("A spawn location should have been set, as if not, it was an error");
        Ok(ActiveGame {
            players: HashSet::new(),
            arena: arena_name.to_string(),
            status: GameStatus::Queuing,
            map_region: arena.map_region,
            death_zone: arena.death_zone,
            spawn,
            spectator: arena.spectator.unwrap_or(spawn),
            lobby: arena.lobby.unwrap_or(spawn),
            min_players: arena.min_players,
            max_players: arena.max_players,
            materials: arena.materials.clone(),
        })
    }

    fn add_player(&mut self, player: Uuid) -> bool {
        self.players.insert(player)
    }

    fn remove_player(&mut self, player: &Uuid) -> bool {
        self.players.remove(player)
    }

    fn is_full(&self) -> bool {
        self.max_players
            .is_some_and(|max| self.players.len() >= max)
    }

    fn broadcast(&self, server: &Server, text_component_factory: impl Fn() -> TextComponent) {
        for uuid in &self.players {
            if let Some(player) = server.get_player_by_uuid(&uuid.to_string()) {
                player.send_system_message(text_component_factory(), false);
            }
        }
    }
}

impl Drop for ActiveGame {
    fn drop(&mut self) {
        // We don't want to double-panic.
        if let Ok(mut data) = SpleefData::get_without_unwrap()
            && let Ok(arena) = data.config.get_arena_mut(&self.arena)
        {
            arena.occupied = false;
        }
    }
}

/// Used for calling a function that prints join/leave messages.
#[derive(Debug, Copy, Clone)]
pub enum JoinOrLeave {
    Join,
    Leave,
}

impl JoinOrLeave {
    pub fn color(&self) -> NamedColor {
        match self {
            JoinOrLeave::Join => NamedColor::Green,
            JoinOrLeave::Leave => NamedColor::Red,
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            JoinOrLeave::Join => "+",
            JoinOrLeave::Leave => "-",
        }
    }

    pub fn message(&self) -> &'static str {
        match self {
            JoinOrLeave::Join => " has joined the game!",
            JoinOrLeave::Leave => " has left the game!",
        }
    }
}

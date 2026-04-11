use pumpkin_plugin_api::server::Player;
use pumpkin_plugin_api::world::BlockPos;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use std::fs::File;
use std::io::{ErrorKind, Read, Write};
use std::num::NonZero;
use std::path::Path;

pub static CONFIG_FILE_NAME: &str = "config.json";

/// A structure to store the current configuration
/// of the plugin that will persist between server restarts.
#[derive(Serialize, Deserialize, Default)]
pub struct Configuration {
    /// Represents possible arenas where the game will take place.
    pub arenas: HashMap<String, Arena>,
}

impl Configuration {
    /// Attempts to save this configuration to disk.
    pub fn save_to_disk(&self, config_path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|error| format!("Could not serialize configuration to JSON: {error}"))?;

        let mut file = File::create(config_path)
            .map_err(|error| format!("Could not create configuration file: {error}"))?;

        file.write_all(json.as_bytes())
            .map_err(|error| format!("Could not write to configuration file: {error}"))?;

        Ok(())
    }

    /// Attempts to load the configuration from disk.
    pub fn load_from_disk(config_path: &Path) -> Result<Configuration, String> {
        match File::open(config_path) {
            Ok(mut file) => {
                let mut json: String = String::new();
                file.read_to_string(&mut json)
                    .map_err(|error| format!("Could not read from configuration file: {error}"))?;

                let config: Configuration = serde_json::from_str(&json).map_err(|error| {
                    format!("Could not deserialize configuration from JSON: {error}")
                })?;

                tracing::info!("Successfully loaded configuration from disk.");

                Ok(config)
            }
            Err(error) => {
                if error.kind() == ErrorKind::NotFound {
                    // Return an empty configuration as there is no file yet
                    // (booted up for the first time)
                    tracing::info!("Configuration file does not exist. Using a default one...");
                    Ok(Configuration::default())
                } else {
                    Err(format!("Could not open configuration file: {error}"))
                }
            }
        }
    }

    /// Attempts to save this configuration to disk and prints the result about it.
    pub fn save_to_disk_and_print(&self, config_path: &Path) {
        if let Err(error) = self.save_to_disk(config_path) {
            tracing::error!("Could not save configuration to disk: {error}");
        }
    }

    /// Attempts to load the configuration from disk and prints the result about it.
    /// Returns an empty configuration if it failed to load.
    pub fn load_from_disk_and_print(config_path: &Path) -> Configuration {
        Self::load_from_disk(config_path).unwrap_or_else(|error| {
            tracing::error!("Could not load configuration from disk: {error}");
            Configuration::default()
        })
    }
}

/// Represents an arena. Up to a maximum
/// of one game can take place at an arena, at a time.
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Arena {
    /// The region compromising the entire arena.
    pub region: Option<Region>,

    /// Represents the floors of breakable blocks in the arena.
    pub floors: Vec<Region>,

    /// Represents an optional death zone.
    /// This is optional because blocks like lava
    /// kill the player anyway.
    pub death_zone: Option<Region>,

    /// Represents the spawn location of the players upon the game starting.
    pub spawn: Option<Location>,

    /// Represents the lobby location of the players before the game starts.
    /// If unspecified, this will default to the spawn point.
    pub lobby: Option<Location>,

    /// Minimum number of players required to start the game.
    /// If [`None`], this is assumed to be 2.
    pub min_players: Option<NonZero<usize>>,

    /// Maximum number of players required to start the game.
    /// If [`None`], this is assumed to be infinity.
    pub max_players: Option<NonZero<usize>>,
}

impl Arena {
    pub fn is_playable(&self) -> bool {
        self.region.is_some() && !self.floors.is_empty() && self.spawn.is_some()
    }

    pub fn min_players(&self) -> usize {
        self.min_players.map(NonZero::get).unwrap_or(2)
    }

    pub fn max_players(&self) -> Option<usize> {
        self.max_players.map(NonZero::get)
    }

    pub fn spawn(&self) -> Option<Location> {
        self.spawn
    }

    pub fn lobby(&self) -> Option<Location> {
        self.lobby.or(self.spawn)
    }
}

/// Represents a three-dimensional rectangular region of blocks.
///
/// Block position `1` contains the minimum of the coordinates,
/// while block position `2` contains the maximum of the coordinates.
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct Region {
    pub x1: i32,
    pub y1: i32,
    pub z1: i32,
    pub x2: i32,
    pub y2: i32,
    pub z2: i32,
}

/// Represents a 3D location which includes the x, y, and z coordinates of the player's
/// position along with their pitch and yaw.
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct Location {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
}

impl Location {
    pub fn from_player(player: &Player) -> Location {
        let (x, y, z) = player.get_position();
        Location {
            x,
            y,
            z,
            yaw: player.get_yaw(),
            pitch: player.get_pitch(),
        }
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:.2}, {:.2}, {:.2} [{:.2}, {:.2}]",
            self.x, self.y, self.z, self.yaw, self.pitch
        )
    }
}

impl Region {
    /// Constructs a new region between two block positions.
    pub fn new(pos1: &BlockPos, pos2: &BlockPos) -> Self {
        let (x1, x2) = Self::min_max(pos1.x, pos2.x);
        let (y1, y2) = Self::min_max(pos1.y, pos2.y);
        let (z1, z2) = Self::min_max(pos1.z, pos2.z);

        Self {
            x1,
            y1,
            z1,
            x2,
            y2,
            z2,
        }
    }

    /// Provides the required `(minimum, maximum)` tuple according
    /// to the values of the coordinates given in an axis.
    fn min_max(coordinate1: i32, coordinate2: i32) -> (i32, i32) {
        if coordinate1 < coordinate2 {
            (coordinate1, coordinate2)
        } else {
            (coordinate2, coordinate1)
        }
    }

    /// Checks if the provided block position is inside the region.
    fn contains(&self, pos: &BlockPos) -> bool {
        self.x1 <= pos.x
            && pos.x <= self.x2
            && self.y1 <= pos.y
            && pos.y <= self.y2
            && self.z1 <= pos.z
            && pos.z <= self.z2
    }
}

use pumpkin_plugin_api::common::BlockPosition;
use pumpkin_plugin_api::server::Player;
use pumpkin_plugin_api::world::BlockPos;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Formatter;

/// Represents an arena. Up to a maximum
/// of one game can take place at an arena, at a time.
#[derive(Serialize, Deserialize, Debug)]
pub struct Arena {
    /// The region compromising the entire arena.
    pub map_region: Option<Region>,

    /// Represents an optional death zone.
    /// This is optional because blocks like lava
    /// kill the player anyway.
    pub death_zone: Option<Region>,

    /// Represents the spawn location of the players upon the game starting.
    pub spawn: Option<Location>,

    /// Represents the lobby location of the players before the game starts.
    /// If unspecified, this will default to the spawn point.
    pub lobby: Option<Location>,

    /// Represents the spectator spawning location of the players when they die.
    pub spectator: Option<Location>,

    /// Minimum number of players required to start the game.
    pub min_players: usize,

    /// Maximum number of players required to start the game.
    /// If [`None`], this is assumed to be infinity.
    pub max_players: Option<usize>,

    /// Materials allowed to be broken for the game.
    /// Strings here are blocks.
    ///
    /// By default, it is set to be snow blocks only
    pub materials: Vec<String>,

    /// Returns whether this arena is locked.
    pub locked: bool,
}

impl Default for Arena {
    fn default() -> Self {
        Self {
            map_region: None,
            death_zone: None,
            spawn: None,
            lobby: None,
            spectator: None,
            min_players: 2,
            max_players: None,
            materials: vec!["minecraft:snow_block".into()],
            locked: false,
        }
    }
}

impl Arena {
    pub fn is_playable(&self) -> bool {
        self.map_region.is_some() && self.spawn.is_some()
    }

    pub fn min_players(&self) -> usize {
        self.min_players
    }

    pub fn max_players(&self) -> Option<usize> {
        // Ensure that max_players returned is never less than min_players
        self.max_players.map(|max| max.max(self.min_players()))
    }

    pub fn spawn(&self) -> Option<Location> {
        self.spawn
    }

    pub fn lobby(&self) -> Option<Location> {
        self.lobby.or(self.spawn)
    }

    pub fn spectator(&self) -> Option<Location> {
        self.spectator.or(self.spawn)
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

impl Region {
    /// Constructs a new region between two block positions.
    pub fn new(pos1: &BlockPosition, pos2: &BlockPosition) -> Self {
        let (x1, x2) = Self::min_max(pos1.0, pos2.0);
        let (y1, y2) = Self::min_max(pos1.1, pos2.1);
        let (z1, z2) = Self::min_max(pos1.2, pos2.2);

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

impl fmt::Display for Region {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}, {}, {} to {}, {}, {}",
            self.x1, self.y1, self.z1, self.x2, self.y2, self.z2
        )
    }
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

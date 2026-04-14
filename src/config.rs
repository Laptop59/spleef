use crate::arena::{Arena, ArenaError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{ErrorKind, Read, Write};
use std::path::Path;

pub static CONFIG_FILE_NAME: &str = "config.json";

/// A structure to store the current configuration
/// of the plugin that will persist between server restarts.
#[derive(Serialize, Deserialize, Default)]
pub struct Configuration {
    arenas: HashMap<String, Arena>,
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

    /// Attempts to create a new arena with the specified name
    /// with the default settings. Returns `true` if it succeeded.
    pub fn add_arena(&mut self, arena: &str) -> Result<(), ArenaError> {
        if self.arenas.contains_key(arena) {
            Err(ArenaError::AlreadyExists(arena.to_string()))
        } else {
            self.arenas.insert(arena.to_string(), Arena::default());
            Ok(())
        }
    }

    /// Attempts to delete an arena with the specified name
    /// with the default settings. Fails if a game is already active on it
    pub fn remove_arena(&mut self, arena: &str) -> Result<(), ArenaError> {
        if !self.arenas.contains_key(arena) {
            Err(ArenaError::NoSuchArena(arena.to_string()))
        } else {
            if self.arenas[arena].occupied {
                Err(ArenaError::Occupied)
            } else {
                self.arenas.remove(arena);
                Ok(())
            }
        }
    }

    /// Returns if an arena exists with the provided name.
    pub fn arena_exists(&self, arena: &str) -> bool {
        self.arenas.contains_key(arena)
    }

    /// Returns a list of all the arenas.
    pub fn list_arenas(&self) -> Vec<(&str, &Arena)> {
        self.arenas
            .iter()
            .map(|(key, arena)| (key.as_str(), arena))
            .collect()
    }

    /// Attempts to get a read-only reference to an arena.
    /// Fails if the arena doesn't exist.
    pub fn get_arena(&self, arena: &str) -> Result<&Arena, ArenaError> {
        self.arenas
            .get(arena)
            .ok_or_else(|| ArenaError::NoSuchArena(arena.to_string()))
    }

    /// Attempts to get a mutable reference to an arena. Fails
    /// if a game is already active on it, or if the arena doesn't exist.
    pub fn get_arena_mut(&mut self, arena: &str) -> Result<&mut Arena, ArenaError> {
        if let Some(arena) = self.arenas.get_mut(arena) {
            if arena.occupied {
                Err(ArenaError::Occupied)
            } else {
                Ok(arena)
            }
        } else {
            Err(ArenaError::NoSuchArena(arena.to_string()))
        }
    }
}

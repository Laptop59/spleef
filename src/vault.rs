use pumpkin_plugin_api::common::ItemStack;
use pumpkin_plugin_api::server::Player;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::{ErrorKind, Read, Write};
use std::path::PathBuf;

/// The vault is a system that saves player's inventories to disk
/// before clearing their inventory and restores it after the game ends or whenever
/// appropriate.
#[derive(Debug, Default)]
pub struct Vault {
    folder_path: PathBuf,
}

impl Vault {
    /// Creates a new vault with reference to the given vault folder.
    pub fn new(folder_path: PathBuf) -> Vault {
        Vault { folder_path }
    }
}

/// Saves items of a particular player.
#[derive(Serialize, Deserialize)]
pub struct PlayerVault {
    items: Vec<Option<SavedItemStack>>,
}

/// A serializable version of the [`ItemStack`] structure.
#[derive(Serialize, Deserialize)]
pub struct SavedItemStack {
    id: String,
    count: u8,
}

impl Into<ItemStack> for SavedItemStack {
    fn into(self) -> ItemStack {
        ItemStack {
            registry_key: self.id,
            count: self.count,
        }
    }
}

impl Into<SavedItemStack> for ItemStack {
    fn into(self) -> SavedItemStack {
        SavedItemStack {
            id: self.registry_key,
            count: self.count,
        }
    }
}

pub static VAULTS_FOLDER_NAME: &str = "vaults";

impl Vault {
    pub fn save_and_clear(&self, player: &Player) -> Result<(), String> {
        let mut vault_path = self.folder_path.clone();
        vault_path.push(player.get_id());

        fs::create_dir_all(&self.folder_path).map_err(|error| error.to_string())?;
        let mut file = File::create_new(vault_path).map_err(|error| error.to_string())?;
        // Serialize all player inventory slots.

        // TODO: Serialize armor slots & off-hand when we can, because right now we cannot.
        // The item stacks we get are only limited to count & ID, no data components.
        // That should be easy to serialize though :)

        let mut items: Vec<Option<SavedItemStack>> = Vec::new();
        for i in 0..36 {
            let stack = player.get_inventory_item(i);
            items.push(stack.map(Into::into));
        }

        let player_vault = PlayerVault { items };

        let json = serde_json::to_string(&player_vault).map_err(|error| error.to_string())?;
        file.write_all(json.as_bytes())
            .map_err(|error| error.to_string())?;

        // Since the file successfully saved, we clear the inventory
        // TODO: Oh wait, we can't yet

        Ok(())
    }

    pub fn load(&self, player: &Player) -> Result<(), String> {
        let mut vault_path = self.folder_path.clone();
        vault_path.push(player.get_id());

        match File::open(&vault_path) {
            Ok(mut file) => {
                let mut json = String::new();
                file.read_to_string(&mut json)
                    .map_err(|error| error.to_string())?;

                let player_vault: PlayerVault =
                    serde_json::from_str(&json).map_err(|error| error.to_string())?;

                drop(file);
                std::fs::remove_file(vault_path).map_err(|error| error.to_string())?;

                // TODO: Set items back when we can

                Ok(())
            }
            Err(error) => {
                if error.kind() == ErrorKind::NotFound {
                    // Ignore it, that's totally fine.
                    Ok(())
                } else {
                    Err(error.to_string())
                }
            }
        }
    }
}

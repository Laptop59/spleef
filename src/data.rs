use crate::command::init_command_tree;
use crate::config::{CONFIG_FILE_NAME, Configuration};
use crate::event_handler::register_event_handlers;
use crate::game::GameManager;
use crate::vault::{VAULTS_FOLDER_NAME, Vault};
use pumpkin_plugin_api::Context;
use pumpkin_plugin_api::permission::{Permission, PermissionDefault};
use pumpkin_plugin_api::text::RgbColor;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, LockResult, Mutex, MutexGuard};

pub const ERROR_COLOR: RgbColor = RgbColor {
    r: 0xFF,
    g: 0x6F,
    b: 0x6F,
};
pub const WARNING_COLOR: RgbColor = RgbColor {
    r: 0xFF,
    g: 0xFF,
    b: 0x6F,
};
pub const OK_COLOR: RgbColor = RgbColor {
    r: 0x6F,
    g: 0xFF,
    b: 0x6F,
};

pub static INSTANCE: LazyLock<Arc<Mutex<SpleefData>>> =
    LazyLock::new(|| Arc::new(Mutex::new(SpleefData::default())));

/// Structure that stores current data of the plugin (the state).
#[derive(Default)]
pub struct SpleefData {
    /// The current configuration of the plugin.
    pub config: Configuration,

    /// The vault of the plugin.
    pub vault: Vault,

    /// Represents the path to the configuration file.
    pub config_file: PathBuf,

    /// Represents the game manager, which managing all games.
    pub game_manager: GameManager,
}

impl SpleefData {
    /// Gets the global instance of this plugin's data.
    /// Should only be used in command handlers or event handlers.
    pub fn get() -> MutexGuard<'static, SpleefData> {
        INSTANCE.lock().expect("Did not expect a poisoned mutex")
    }

    /// Gets the global instance of this plugin's data without expecting unpoisoned mutexes.
    pub fn get_without_unwrap() -> LockResult<MutexGuard<'static, SpleefData>> {
        INSTANCE.lock()
    }

    pub fn load(&mut self, context: &Context) -> pumpkin_plugin_api::Result<()> {
        self.config_file = {
            let folder = context.get_data_folder();
            let mut path: PathBuf = folder.into();
            path.push(CONFIG_FILE_NAME);
            path
        };

        self.config = Configuration::load_from_disk_and_print(&self.config_file);
        self.vault = Vault::new({
            let folder = context.get_data_folder();
            let mut path: PathBuf = folder.into();
            path.push(VAULTS_FOLDER_NAME);
            path
        });

        context.register_permission(&Permission {
            // This has to have the same name space as provided in your PluginMetadata
            node: "spleef:main".to_string(),
            description: "Minimum permission required to access any spleef subcommand".to_string(),
            default: PermissionDefault::Allow,
            children: Vec::new(),
        })?;
        context.register_command(init_command_tree(), "spleef:main");

        register_event_handlers(context)?;

        Ok(())
    }

    pub fn unload(&mut self) -> pumpkin_plugin_api::Result<()> {
        self.config.save_to_disk_and_print(&self.config_file);

        Ok(())
    }
}

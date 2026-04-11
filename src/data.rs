use crate::command::init_command_tree;
use crate::config::{CONFIG_FILE_NAME, Configuration};
use crate::game::ActiveGame;
use pumpkin_plugin_api::Context;
use pumpkin_plugin_api::permission::{Permission, PermissionDefault};
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, Mutex, MutexGuard};

pub static INSTANCE: LazyLock<Arc<Mutex<SpleefData>>> =
    LazyLock::new(|| Arc::new(Mutex::new(SpleefData::default())));

/// Structure that stores current data of the plugin (the state).
#[derive(Default)]
pub struct SpleefData {
    /// The current configuration of the plugin.
    pub config: Configuration,

    /// Represents the path to the configuration file.
    pub config_file: PathBuf,

    /// Represents the currently active games.
    pub active_games: Vec<ActiveGame>,
}

impl SpleefData {
    /// Gets the global instance of this plugin's data.
    pub fn get() -> MutexGuard<'static, SpleefData> {
        INSTANCE.lock().unwrap()
    }

    pub fn load(&mut self, context: &Context) -> pumpkin_plugin_api::Result<()> {
        self.config_file = {
            let folder = context.get_data_folder();
            let mut path: PathBuf = folder.into();
            path.push(CONFIG_FILE_NAME);
            path
        };

        self.config = Configuration::load_from_disk_and_print(&self.config_file);

        context.register_permission(&Permission {
            // This has to have the same name space as provided in your PluginMetadata
            node: "spleef:main".to_string(),
            description: "Default permission to access the non-admin spleef subcommands"
                .to_string(),
            default: PermissionDefault::Allow,
            children: Vec::new(),
        })?;
        context.register_command(init_command_tree(), "spleef:main");

        Ok(())
    }

    pub fn unload(&mut self) -> pumpkin_plugin_api::Result<()> {
        self.config.save_to_disk_and_print(&self.config_file);

        Ok(())
    }
}

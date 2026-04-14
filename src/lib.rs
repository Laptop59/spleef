mod arena;
mod command;
mod config;
mod data;
mod event_handler;
mod game;

use crate::data::SpleefData;
use pumpkin_plugin_api::{Context, Plugin, PluginMetadata};
use tracing::*;

struct SpleefPlugin;

impl Plugin for SpleefPlugin {
    fn new() -> Self {
        SpleefPlugin
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "spleef".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            authors: vec!["Laptop59".into()],
            description: "A simple spleef plugin".into(),
        }
    }

    fn on_load(&mut self, context: Context) -> pumpkin_plugin_api::Result<()> {
        SpleefData::get().load(&context)?;

        info!("Loaded Spleef!");
        Ok(())
    }

    fn on_unload(&mut self, _context: Context) -> pumpkin_plugin_api::Result<()> {
        SpleefData::get().unload()?;

        info!("Unloaded Spleef!");
        Ok(())
    }
}

pumpkin_plugin_api::register_plugin!(SpleefPlugin);

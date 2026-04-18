use crate::arena::{ArenaError, Location};
use crate::command::ARG_ARENA;
use crate::data::{OK_COLOR, SpleefData};
use pumpkin_plugin_api::command::{CommandError, CommandNode, CommandSender, ConsumedArgs};
use pumpkin_plugin_api::command_wit::{Arg, ArgumentType, StringType};
use pumpkin_plugin_api::commands::CommandHandler;
use pumpkin_plugin_api::common::NamedColor;
use pumpkin_plugin_api::server::Server;
use pumpkin_plugin_api::text::TextComponent;

struct SpawnCommandExecutor {
    clear: bool,
}

impl CommandHandler for SpawnCommandExecutor {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> pumpkin_plugin_api::Result<i32, CommandError> {
        let Arg::Simple(arena_name) = args.get_value(ARG_ARENA) else {
            return Err(CommandError::InvalidConsumption(Some(
                "Could not parse arena arg".to_string(),
            )));
        };

        let mut data = SpleefData::get();
        let arena = data
            .config
            .get_arena_mut(&arena_name)
            .map_err(ArenaError::command_error)?;

        if self.clear {
            arena.spawn.clear();
            sender.send_message(
                TextComponent::text("Cleared all spawn locations.").color_rgb(OK_COLOR),
            );
        } else {
            let Some(player) = sender.as_player() else {
                let error = TextComponent::translate("permissions.requires.player", vec![]);
                return Err(CommandError::CommandFailed(error));
            };

            let location = Location::from_player(&player);
            arena.spawn.push(location.clone());

            sender.send_message(
                TextComponent::text("Added spawn location ")
                    .color_rgb(OK_COLOR)
                    .add_child({
                        let text2 = TextComponent::text(&location.to_string());
                        text2.color_named(NamedColor::White);
                        text2
                    })
                    .add_text("."),
            );
        }

        Ok(arena.spawn.len() as i32)
    }
}

pub fn spawn() -> CommandNode {
    let node = CommandNode::literal("spawn");
    node.then({
        let arena = CommandNode::argument(ARG_ARENA, &ArgumentType::String(StringType::SingleWord));
        arena.then(CommandNode::literal("add").execute(SpawnCommandExecutor { clear: false }));
        arena.then(CommandNode::literal("clear").execute(SpawnCommandExecutor { clear: true }));
        arena
    });
    node
}

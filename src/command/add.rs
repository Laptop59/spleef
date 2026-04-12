use crate::command::ARG_ARENA;
use crate::config::ArenaError;
use crate::data::SpleefData;
use pumpkin_plugin_api::command::{CommandError, CommandNode, CommandSender, ConsumedArgs};
use pumpkin_plugin_api::command_wit::{Arg, ArgumentType, StringType};
use pumpkin_plugin_api::commands::CommandHandler;
use pumpkin_plugin_api::common::NamedColor;
use pumpkin_plugin_api::server::Server;
use pumpkin_plugin_api::text::TextComponent;

struct AddCommandExecutor;

impl CommandHandler for AddCommandExecutor {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> pumpkin_plugin_api::Result<i32, CommandError> {
        let Arg::Simple(arena) = args.get_value(ARG_ARENA) else {
            return Err(CommandError::InvalidConsumption(Some(
                "Could not parse arena arg".to_string(),
            )));
        };

        let mut data = SpleefData::get();
        data.config
            .create_arena(&arena)
            .map_err(ArenaError::command_error)?;

        let text = TextComponent::text(&format!("Successfully created the arena '{arena}'."));
        text.color_named(NamedColor::Green);
        sender.send_message(text);
        Ok(1)
    }
}

pub fn add() -> CommandNode {
    let node = CommandNode::literal("add");
    node.then(
        CommandNode::argument(ARG_ARENA, &ArgumentType::String(StringType::SingleWord))
            .execute(AddCommandExecutor),
    );
    node
}

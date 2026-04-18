use crate::arena::ArenaError;
use crate::command::ARG_ARENA;
use crate::data::OK_COLOR;
use crate::data::SpleefData;
use pumpkin_plugin_api::command::{CommandError, CommandNode, CommandSender, ConsumedArgs};
use pumpkin_plugin_api::command_wit::{Arg, ArgumentType, StringType};
use pumpkin_plugin_api::commands::CommandHandler;
use pumpkin_plugin_api::server::Server;
use pumpkin_plugin_api::text::TextComponent;

struct RemoveCommandExecutor;

impl CommandHandler for RemoveCommandExecutor {
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
            .remove_arena(&arena)
            .map_err(ArenaError::command_error)?;

        sender.send_message(
            TextComponent::text(&format!("Successfully removed the arena {arena}."))
                .color_rgb(OK_COLOR),
        );
        Ok(1)
    }
}

pub fn remove() -> CommandNode {
    let node = CommandNode::literal("remove");
    node.then(
        CommandNode::argument(ARG_ARENA, &ArgumentType::String(StringType::SingleWord))
            .execute(RemoveCommandExecutor),
    );
    node
}

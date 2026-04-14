use crate::arena::ArenaError;
use crate::command::ARG_ARENA;
use crate::data::SpleefData;
use pumpkin_plugin_api::command::{CommandError, CommandNode, CommandSender, ConsumedArgs};
use pumpkin_plugin_api::command_wit::{Arg, ArgumentType, StringType};
use pumpkin_plugin_api::commands::CommandHandler;
use pumpkin_plugin_api::server::Server;

struct StatusCommandExecutor;

impl CommandHandler for StatusCommandExecutor {
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

        let (errors, warnings) = SpleefData::get()
            .config
            .get_arena(&arena)
            .map_err(ArenaError::command_error)?
            .send_errors_and_warnings(&sender);

        Ok((errors + warnings) as i32)
    }
}

pub fn status() -> CommandNode {
    let node = CommandNode::literal("status");
    node.then(
        CommandNode::argument(ARG_ARENA, &ArgumentType::String(StringType::SingleWord))
            .execute(StatusCommandExecutor),
    );
    node
}

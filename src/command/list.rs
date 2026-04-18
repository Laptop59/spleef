use crate::data::{ERROR_COLOR, OK_COLOR, SpleefData, WARNING_COLOR};
use pumpkin_plugin_api::command::{CommandError, CommandNode, CommandSender, ConsumedArgs};
use pumpkin_plugin_api::commands::CommandHandler;
use pumpkin_plugin_api::server::Server;
use pumpkin_plugin_api::text::TextComponent;
struct ListCommandExecutor;

impl CommandHandler for ListCommandExecutor {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> pumpkin_plugin_api::Result<i32, CommandError> {
        sender.send_message(
            TextComponent::text("List of registered spleef arenas:").color_rgb(OK_COLOR),
        );

        let data = SpleefData::get();
        let list = data.config.list_arenas();

        let len = list.len();
        for (name, arena) in list {
            sender.send_message({
                let text = TextComponent::text(&format!("• {name}"));
                let errors = arena.errors();
                let warnings = arena.warnings();

                text.click_suggest_command(&format!("/spleef status {name}"))
                    .hover_show_text(TextComponent::text(
                        "Click to copy the status command for this arena.",
                    ))
                    .color_rgb({
                        if !errors.is_empty() {
                            ERROR_COLOR
                        } else if !warnings.is_empty() {
                            WARNING_COLOR
                        } else {
                            OK_COLOR
                        }
                    })
            });
        }

        Ok(len as i32)
    }
}

pub fn list() -> CommandNode {
    CommandNode::literal("list").execute(ListCommandExecutor)
}

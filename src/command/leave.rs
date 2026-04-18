use crate::command::ARG_TARGET;
use crate::data::SpleefData;
use pumpkin_plugin_api::command::{CommandError, CommandNode, CommandSender, ConsumedArgs};
use pumpkin_plugin_api::command_wit::{Arg, ArgumentType};
use pumpkin_plugin_api::commands::CommandHandler;
use pumpkin_plugin_api::common::NamedColor;
use pumpkin_plugin_api::server::Server;
use pumpkin_plugin_api::text::TextComponent;
use std::ops::DerefMut;
use uuid::Uuid;

struct LeaveCommandExecutor {
    sender_is_target: bool,
}

impl CommandHandler for LeaveCommandExecutor {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> pumpkin_plugin_api::Result<i32, CommandError> {
        let players = if self.sender_is_target {
            if let Some(player) = sender.as_player() {
                vec![player]
            } else {
                let error = TextComponent::translate("permissions.requires.player", vec![]);
                return Err(CommandError::CommandFailed(error));
            }
        } else {
            let Arg::Players(players) = args.get_value(ARG_TARGET) else {
                return Err(CommandError::InvalidConsumption(Some(
                    "Could not parse target arg".to_string(),
                )));
            };
            players
        };

        let mut data = SpleefData::get();
        let data = data.deref_mut();

        let mut successes: i32 = 0;
        for player in players {
            data.game_manager.remove_player(
                &Uuid::parse_str(&player.get_id())
                    .expect("Pumpkin did not return a valid UUID string"),
                &server,
            );

            successes += 1;
        }

        sender.send_message({
            let text = if self.sender_is_target && successes == 1 {
                TextComponent::text("You have left the game.")
            } else {
                TextComponent::text(&format!("Made {successes} players leave the game."))
            };

            text.color_named(NamedColor::Red);
            text
        });

        Ok(successes)
    }
}

pub fn leave() -> CommandNode {
    let node = CommandNode::literal("leave");
    node.then(
        CommandNode::argument(ARG_TARGET, &ArgumentType::Players).execute(LeaveCommandExecutor {
            sender_is_target: false,
        }),
    );
    node.execute(LeaveCommandExecutor {
        sender_is_target: true,
    })
}

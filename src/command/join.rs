use crate::arena::ArenaError;
use crate::command::{ARG_ARENA, ARG_TARGET};
use crate::data::SpleefData;
use pumpkin_plugin_api::command::{CommandError, CommandNode, CommandSender, ConsumedArgs};
use pumpkin_plugin_api::command_wit::{Arg, ArgumentType, StringType};
use pumpkin_plugin_api::commands::CommandHandler;
use pumpkin_plugin_api::server::Server;
use pumpkin_plugin_api::text::TextComponent;
use std::ops::DerefMut;
use pumpkin_plugin_api::common::NamedColor;
use uuid::Uuid;

struct JoinCommandExecutor {
    sender_is_target: bool,
}

impl CommandHandler for JoinCommandExecutor {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> pumpkin_plugin_api::Result<i32, CommandError> {
        let Arg::Simple(arena) = args.get_value(ARG_ARENA) else {
            return Err(CommandError::InvalidConsumption(Some(
                "Could not parse arena arg".to_string(),
            )));
        };

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

        if !data.game_manager.has(&arena) {
            data.game_manager
                .create_new(&mut data.config, &arena)
                .map_err(ArenaError::command_error)?;
        }

        let mut successes: i32 = 0;
        for player in players {
            let join_result = data.game_manager
                .join_player(
                    &arena,
                    Uuid::parse_str(&player.get_id())
                        .expect("Pumpkin did not return a valid UUID string"),
                    &server,
                );

            if let Err(error) = join_result {
                sender.send_message(error.text_component());
            }

            successes += 1;
        }

        sender.send_message({
            let text = if self.sender_is_target && successes == 1 {
                TextComponent::text("You have joined the game!")
            } else {
                TextComponent::text(&format!("Made {successes} players join the game."))
            };

            text.color_named(NamedColor::Green);
            text
        });

        Ok(successes)
    }
}

pub fn join() -> CommandNode {
    let node = CommandNode::literal("join");
    node.then({
        let node = CommandNode::argument(ARG_ARENA, &ArgumentType::String(StringType::SingleWord))
            .execute(JoinCommandExecutor {
                sender_is_target: true,
            });

        node.then(
            CommandNode::argument(ARG_TARGET, &ArgumentType::Players).execute(
                JoinCommandExecutor {
                    sender_is_target: false,
                },
            ),
        );

        node
    });
    node
}

use crate::config::{Arena, Location};
use crate::data::SpleefData;
use pumpkin_plugin_api::command::{
    Command, CommandError, CommandNode, CommandSender, ConsumedArgs,
};
use pumpkin_plugin_api::command_wit::{Arg, ArgumentType, StringType};
use pumpkin_plugin_api::commands::CommandHandler;
use pumpkin_plugin_api::common::NamedColor;
use pumpkin_plugin_api::server::Server;
use pumpkin_plugin_api::text::TextComponent;

pub static ARG_ARENA: &str = "arena";

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
        if data.config.arenas.contains_key(&arena) {
            Err(CommandError::CommandFailed(TextComponent::text(
                "This arena already exists!",
            )))
        } else {
            data.config.arenas.insert(arena.clone(), Arena::default());
            let text = TextComponent::text(&format!("Successfully created the arena '{arena}'."));
            text.color_named(NamedColor::Green);
            sender.send_message(text);
            Ok(1)
        }
    }
}

enum SetLocationProperty {
    Spawn,
    Lobby,
}

struct SetLocationCommandExecutor(SetLocationProperty);

impl CommandHandler for SetLocationCommandExecutor {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> pumpkin_plugin_api::Result<i32, CommandError> {
        let Some(player) = sender.as_player() else {
            let error = TextComponent::translate("permissions.requires.player", vec![]);
            return Err(CommandError::CommandFailed(error));
        };

        let Arg::Simple(arena_name) = args.get_value(ARG_ARENA) else {
            return Err(CommandError::InvalidConsumption(Some(
                "Could not parse arena arg".to_string(),
            )));
        };

        let mut data = SpleefData::get();
        let Some(arena) = data.config.arenas.get_mut(&arena_name) else {
            return Err(CommandError::CommandFailed(TextComponent::text(&format!(
                "No arena '{arena_name}' exists!"
            ))));
        };

        let (location, name) = match self.0 {
            SetLocationProperty::Spawn => (&mut arena.spawn, "spawn"),
            SetLocationProperty::Lobby => (&mut arena.lobby, "lobby"),
        };

        // Take the player's location and set it
        let new_location = Location::from_player(&player);
        *location = Some(new_location);

        sender.send_message({
            let text= TextComponent::text(&format!(
                "Set the {name} location to "
            ));
            text.color_named(NamedColor::Green);
            text.add_child({
                let text2 = TextComponent::text(&new_location.to_string());
                text2.color_named(NamedColor::White);
                text2
            });
            text.add_text(".");
            text
        });
        Ok(1)
    }
}

pub fn init_command_tree() -> Command {
    let names = ["spleef".to_string()];
    let description = "Command for the Spleef plugin!";

    let command = Command::new(&names, description);
    command.then(add());
    command.then(set());
    command
}

fn add() -> CommandNode {
    let node = CommandNode::literal("add");
    node.then(
        CommandNode::argument(ARG_ARENA, &ArgumentType::String(StringType::SingleWord))
            .execute(AddCommandExecutor),
    );
    node
}

fn set() -> CommandNode {
    let node = CommandNode::literal("set");
    let set = CommandNode::argument(ARG_ARENA, &ArgumentType::String(StringType::SingleWord));
    set.then(
        CommandNode::literal("spawn")
            .execute(SetLocationCommandExecutor(SetLocationProperty::Spawn)),
    );
    set.then(
        CommandNode::literal("lobby")
            .execute(SetLocationCommandExecutor(SetLocationProperty::Lobby)),
    );
    node.then(set);
    node
}

use crate::arena::ArenaError;
use crate::arena::{Arena, Location};
use crate::command::{ARG_ARENA, ARG_FROM, ARG_TO, ARG_VALUE, parse_generic_region};
use crate::data::OK_COLOR;
use crate::data::SpleefData;
use pumpkin_plugin_api::command::{CommandError, CommandNode, CommandSender, ConsumedArgs};
use pumpkin_plugin_api::command_wit::{Arg, ArgumentType, Number, StringType};
use pumpkin_plugin_api::commands::CommandHandler;
use pumpkin_plugin_api::common::NamedColor;
use pumpkin_plugin_api::server::Server;
use pumpkin_plugin_api::text::TextComponent;
use std::sync::MutexGuard;

#[derive(Copy, Clone, Debug)]
enum SetLocationProperty {
    Spawn,
    Lobby,
    Spectator,
}

struct SetLocationCommandExecutor(SetLocationProperty);

impl CommandHandler for SetLocationCommandExecutor {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> pumpkin_plugin_api::Result<i32, CommandError> {
        let mut data = SpleefData::get();
        let arena = parse_arena(&mut data, &args)?;

        let Some(player) = sender.as_player() else {
            let error = TextComponent::translate("permissions.requires.player", vec![]);
            return Err(CommandError::CommandFailed(error));
        };

        let (location, name) = match self.0 {
            SetLocationProperty::Spawn => (&mut arena.spawn, "spawn"),
            SetLocationProperty::Lobby => (&mut arena.lobby, "lobby"),
            SetLocationProperty::Spectator => (&mut arena.spectator, "spectator"),
        };

        let new_location = Location::from_player(&player);
        *location = Some(new_location);

        sender.send_message({
            let text = TextComponent::text(&format!("Set the {name} location to "));
            text.color_rgb(OK_COLOR);
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

#[derive(Copy, Clone, Debug)]
enum SetIntegerProperty {
    MinPlayers,
    MaxPlayers,
}

struct SetIntegerCommandExecutor {
    property: SetIntegerProperty,
    value_specified: bool,
}

impl CommandHandler for SetIntegerCommandExecutor {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> pumpkin_plugin_api::Result<i32, CommandError> {
        let value: Option<usize> = if self.value_specified {
            let Arg::Num(value) = args.get_value(ARG_VALUE) else {
                return Err(CommandError::InvalidConsumption(Some(
                    "Could not parse value arg".to_string(),
                )));
            };
            if let Ok(Number::Int64(value)) = value
                && let Ok(result) = value.try_into()
            {
                Some(result)
            } else {
                return Err(CommandError::CommandFailed(TextComponent::text(
                    "Integer provided was not in the required bounds.",
                )));
            }
        } else {
            None
        };

        let mut data = SpleefData::get();
        let arena = parse_arena(&mut data, &args)?;

        let (name, default_value) = match self.property {
            SetIntegerProperty::MinPlayers => {
                arena.min_players = value.unwrap_or(2);
                ("minimum number of players required", "2")
            }
            SetIntegerProperty::MaxPlayers => {
                arena.max_players = value;
                ("maximum number of players allowed", "∞")
            }
        };

        if let Some(value) = value {
            sender.send_message({
                let text = TextComponent::text(&format!("Set the {name} to "));
                text.color_rgb(OK_COLOR);
                text.add_child({
                    let text2 = TextComponent::text(&value.to_string());
                    text2.color_named(NamedColor::White);
                    text2
                });
                text.add_text(".");
                text
            });
        } else {
            sender.send_message({
                let text = TextComponent::text(&format!("Reset the {name} to its default value "));
                text.color_rgb(OK_COLOR);
                text.add_child({
                    let text2 = TextComponent::text(default_value);
                    text2.color_named(NamedColor::White);
                    text2
                });
                text.add_text(".");
                text
            });
        }

        Ok(1)
    }
}

#[derive(Copy, Clone, Debug)]
enum SetRegionProperty {
    MapRegion,
    DeathZone,
}

struct SetRegionCommandExecutor {
    property: SetRegionProperty,
    value_specified: bool,
}

impl CommandHandler for SetRegionCommandExecutor {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> pumpkin_plugin_api::Result<i32, CommandError> {
        let region = if self.value_specified {
            Some(parse_generic_region(&args)?)
        } else {
            None
        };

        let mut data = SpleefData::get();
        let arena = parse_arena(&mut data, &args)?;

        let (property, name) = match self.property {
            SetRegionProperty::MapRegion => (&mut arena.map_region, "map region"),
            SetRegionProperty::DeathZone => (&mut arena.death_zone, "death zone"),
        };

        *property = region;

        if let Some(region) = region {
            sender.send_message({
                let text = TextComponent::text(&format!("Set the {name} to the region "));
                text.color_rgb(OK_COLOR);
                text.add_child({
                    let text2 = TextComponent::text(&region.to_string());
                    text2.color_named(NamedColor::White);
                    text2
                });
                text.add_text(".");
                text
            });
        } else {
            sender.send_message({
                let text = TextComponent::text(&format!("Cleared the {name}."));
                text.color_rgb(OK_COLOR);
                text
            });
        }

        Ok(1)
    }
}

struct SetMaterialCommandExecutor {
    blocks_specified: usize,
}

impl CommandHandler for SetMaterialCommandExecutor {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> pumpkin_plugin_api::Result<i32, CommandError> {
        let mut data = SpleefData::get();
        let arena = parse_arena(&mut data, &args)?;

        let Some(block_states): Option<Vec<String>> = (1..=self.blocks_specified)
            .map(|i| format!("material{i}"))
            .map(|arg_key| args.get_value(&arg_key))
            .map(|arg| match arg {
                Arg::Block(id) => Some(id),
                _ => None,
            })
            // If any one `None` is found, the entire collection becomes `None`.
            .collect()
        else {
            return Err(CommandError::InvalidConsumption(Some(
                "Could not parse block state argument(s)".to_string(),
            )));
        };

        arena.materials = block_states;

        sender.send_message({
            let text = if self.blocks_specified == 1 {
                TextComponent::text("Set the materials used for the arena to the block ")
            } else {
                TextComponent::text("Set the materials used for the arena to the blocks ")
            };
            text.color_rgb(OK_COLOR);
            text.add_child({
                let text2 = TextComponent::text(&arena.materials.join(", "));
                text2.color_named(NamedColor::White);
                text2
            });
            text.add_text(".");
            text
        });

        Ok(self.blocks_specified as i32)
    }
}

/// Returns the parsed arena name if it is possible to obtain a mutable reference to it.
fn parse_arena<'a>(
    data: &'a mut MutexGuard<'static, SpleefData>,
    args: &ConsumedArgs,
) -> pumpkin_plugin_api::Result<&'a mut Arena, CommandError> {
    let Arg::Simple(arena_name) = args.get_value(ARG_ARENA) else {
        return Err(CommandError::InvalidConsumption(Some(
            "Could not parse arena arg".to_string(),
        )));
    };

    data.config
        .get_arena_mut(&arena_name)
        .map_err(ArenaError::command_error)
}

pub fn set() -> CommandNode {
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
    set.then(
        CommandNode::literal("spectator")
            .execute(SetLocationCommandExecutor(SetLocationProperty::Spectator)),
    );
    set.then(create_set_integer_node(
        "min_players",
        SetIntegerProperty::MinPlayers,
        Some(2),
        None,
    ));
    set.then(create_set_integer_node(
        "max_players",
        SetIntegerProperty::MaxPlayers,
        Some(2),
        None,
    ));
    set.then(create_set_region_node(
        "map_region",
        SetRegionProperty::MapRegion,
    ));
    set.then(create_set_region_node(
        "death_zone",
        SetRegionProperty::DeathZone,
    ));
    set.then(create_set_material_node());
    node.then(set);
    node
}

fn create_set_integer_node(
    name: &str,
    property: SetIntegerProperty,
    min: Option<i64>,
    max: Option<i64>,
) -> CommandNode {
    let node = CommandNode::literal(name);
    node.then(
        CommandNode::argument(ARG_VALUE, &ArgumentType::Long((min, max))).execute(
            SetIntegerCommandExecutor {
                property,
                value_specified: true,
            },
        ),
    );
    node.execute(SetIntegerCommandExecutor {
        property,
        value_specified: false,
    })
}

fn create_set_region_node(name: &str, property: SetRegionProperty) -> CommandNode {
    let node = CommandNode::literal(name);
    node.then({
        let node = CommandNode::argument(ARG_FROM, &ArgumentType::BlockPos);
        node.then(
            CommandNode::argument(ARG_TO, &ArgumentType::BlockPos).execute(
                SetRegionCommandExecutor {
                    property,
                    value_specified: true,
                },
            ),
        );
        node
    });
    node.execute(SetRegionCommandExecutor {
        property,
        value_specified: false,
    })
}

fn create_set_material_node() -> CommandNode {
    fn material_node(ordinal: usize) -> CommandNode {
        // Maximum allowed is 16 (this is arbitrary by the way)
        //
        // I just don't see a reason why a person would need more
        // than 16 different types of blocks for some floors
        //
        // Also, this way is a pretty hacky way :)

        let node = CommandNode::argument(&format!("material{ordinal}"), &ArgumentType::BlockState);
        if ordinal < 16 {
            node.then(material_node(ordinal + 1));
        }
        node.execute(SetMaterialCommandExecutor {
            blocks_specified: ordinal,
        })
    }

    let node = CommandNode::literal("material");
    node.then(material_node(1));
    node
}

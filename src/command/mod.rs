use crate::arena::Region;
use crate::command::add::add;
use crate::command::set::set;
use pumpkin_plugin_api::command::{Command, CommandError, ConsumedArgs};
use pumpkin_plugin_api::command_wit::Arg;

mod add;
mod set;

pub static ARG_ARENA: &str = "arena";
pub static ARG_VALUE: &str = "value";
pub static ARG_FROM: &str = "from";
pub static ARG_TO: &str = "to";

pub fn parse_generic_region(
    args: &ConsumedArgs,
) -> pumpkin_plugin_api::Result<Region, CommandError> {
    let Arg::BlockPos(from) = args.get_value(ARG_FROM) else {
        return Err(CommandError::InvalidConsumption(Some(
            "Could not parse block position from".to_string(),
        )));
    };
    let Arg::BlockPos(to) = args.get_value(ARG_TO) else {
        return Err(CommandError::InvalidConsumption(Some(
            "Could not parse block position to".to_string(),
        )));
    };
    Ok(Region::new(&from, &to))
}

pub fn init_command_tree() -> Command {
    let names = ["spleef".to_string()];
    let description = "Command for the Spleef plugin!";

    let command = Command::new(&names, description);
    command.then(add());
    command.then(set());
    command
}

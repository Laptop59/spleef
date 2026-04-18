use crate::arena::Region;
use crate::command::add::add;
use crate::command::join::join;
use crate::command::leave::leave;
use crate::command::list::list;
use crate::command::remove::remove;
use crate::command::set::set;
use crate::command::spawn::spawn;
use crate::command::status::status;
use pumpkin_plugin_api::command::{Command, CommandError, ConsumedArgs};
use pumpkin_plugin_api::command_wit::Arg;

mod add;
mod join;
mod leave;
mod list;
mod remove;
mod set;
mod spawn;
mod status;

pub static ARG_ARENA: &str = "arena";
pub static ARG_VALUE: &str = "value";
pub static ARG_FROM: &str = "from";
pub static ARG_TO: &str = "to";
pub static ARG_TARGET: &str = "target";

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
    command.then(join());
    command.then(list());
    command.then(leave());
    command.then(set());
    command.then(spawn());
    command.then(status());
    command.then(remove());
    command
}

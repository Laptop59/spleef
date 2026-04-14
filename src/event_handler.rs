use crate::data::SpleefData;
use pumpkin_plugin_api::events::{EventData, EventHandler, EventPriority, PlayerLeaveEvent};
use pumpkin_plugin_api::{Context, Server};
use uuid::Uuid;

struct PlayerLeaveHandler;
impl EventHandler<PlayerLeaveEvent> for PlayerLeaveHandler {
    fn handle(
        &self,
        server: Server,
        event: EventData<PlayerLeaveEvent>,
    ) -> EventData<PlayerLeaveEvent> {
        let mut data = SpleefData::get();
        let uuid = Uuid::parse_str(&event.player.get_id())
            .expect("Pumpkin did not return a valid UUID string");
        data.game_manager.remove_player(&uuid, &server);

        event
    }
}

pub fn register_event_handlers(context: &Context) -> pumpkin_plugin_api::Result<()> {
    context.register_event_handler(PlayerLeaveHandler, EventPriority::Normal, false)?;

    Ok(())
}

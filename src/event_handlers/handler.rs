use crate::protos::events::bus_event::Event;

pub trait EventHandler {
    fn init(&mut self, _conn: &mut postgres::Client) {}
    fn handle(
        &mut self,
        ctx: &crate::event_handlers::context::InsertContext,
        conn: &mut postgres::Client,
        be: &Event,
    ) -> anyhow::Result<()>;
    fn flush(&mut self, conn: &mut postgres::Client) -> anyhow::Result<()>;
}

use crate::protos::events::bus_event::Event;

pub trait EventHandler {
    fn handle(
        &mut self,
        ctx: &crate::event_handlers::context::InsertContext,
        conn: &mut postgres::Client,
        be: &Event,
    ) -> std::io::Result<()>;
    fn flush(&mut self, conn: &mut postgres::Client);
}

use crate::event_handlers::handler::EventHandler;
use crate::event_handlers::InsertContext;
use crate::events::bus_event::Event;
use crate::protos::events;

pub struct Inserter<'a> {
    pub conn: postgres::Client,
    ctx: InsertContext,
    handlers: &'a mut [&'a mut dyn EventHandler],
}

impl Inserter<'_> {
    pub fn new<'b>(
        conn: postgres::Client,
        handlers: &'b mut [&'b mut dyn EventHandler],
    ) -> Inserter<'b> {
        return Inserter {
            ctx: InsertContext::new(),
            handlers: handlers,
            conn: conn,
        };
    }

    pub fn handle_bus_event(&mut self, be: &events::BusEvent) -> std::io::Result<()> {
        self.ctx.update_from_event(be);
        match be.event.as_ref() {
            Option::Some(Event::TimeUpdate(_)) => self.flush(),
            Option::None => Ok(()),
            _ => Ok(()),
        }
        .unwrap();

        if let Some(event) = be.event.as_ref() {
            for handler in self.handlers.iter_mut() {
                handler.handle(&self.ctx, &mut self.conn, event)?;
            }
        }
        Ok(())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        for handler in self.handlers.iter_mut() {
            handler.flush(&mut self.conn);
        }
        Ok(())
    }
}

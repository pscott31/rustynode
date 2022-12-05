use super::pending::Pending;
use crate::event_handlers::handler::EventHandler;
use crate::event_handlers::InsertContext;
use crate::events::bus_event::Event;
use crate::protos::events;
use anyhow::{Context, Result};

pub struct Inserter {
    pub conn: postgres::Client,
    ctx: InsertContext,
    pending: Pending,
}

impl Inserter {
    pub fn new(conn: postgres::Client) -> Inserter {
        return Inserter {
            ctx: InsertContext::new(),
            pending: Pending::default(),
            conn: conn,
        };
    }

    pub fn handle_bus_event(&mut self, be: &events::BusEvent) -> anyhow::Result<()> {
        self.ctx.update_from_event(be);
        match be.event.as_ref() {
            Option::Some(Event::TimeUpdate(_)) => self.flush()?,
            Option::Some(Event::LedgerMovements(e)) => e.handle(&self.ctx, &mut self.pending)?,
            Option::Some(Event::Account(e)) => e.handle(&self.ctx, &mut self.pending)?,

            _ => (),
        };
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        self.pending
            .flush(&mut self.conn)
            .context("writing block updates to db")
    }
}

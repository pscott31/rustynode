use crate::event_handlers::EventHandler;
use crate::hex_id::HexID;
use crate::protos::events;
use crate::protos::vega;
use crate::utils::account_id_from_details;
use std::str::FromStr;
use std::time::SystemTime;

use postgres_macros::{PgCopyIn, PgTypes};
use postgres_macros_derive::{PgCopyIn, PgTypes};

#[derive(Debug, PgTypes, PgCopyIn)]
struct Ledger {
    account_from_id: HexID,
    account_to_id: HexID,
    quantity: rust_decimal::Decimal,
    #[postgres = "type"]
    type_: vega::TransferType,
    ledger_entry_time: SystemTime,
    transfer_time: SystemTime,
    vega_time: SystemTime,
    tx_hash: HexID,
}

pub struct LedgerEventHandler {
    pending: Vec<Ledger>,
}

impl LedgerEventHandler {
    pub fn new() -> LedgerEventHandler {
        return LedgerEventHandler { pending: vec![] };
    }
}

impl EventHandler for LedgerEventHandler {
    fn init(&mut self, _conn: &mut postgres::Client) {}
    fn handle(
        &mut self,
        ctx: &crate::event_handlers::InsertContext,
        _conn: &mut postgres::Client,
        e: &events::bus_event::Event,
    ) -> std::io::Result<()> {
        let events::bus_event::Event::LedgerMovements(e) = e else {
            return Ok(())
        };

        for lm in e.ledger_movements.iter() {
            for le in &lm.entries {
                let ts =
                    std::time::UNIX_EPOCH + std::time::Duration::from_nanos(le.timestamp as u64);
                let le_time = ctx.vega_time
                    + time::Duration::MICROSECOND
                        .checked_mul(self.pending.len().try_into().unwrap())
                        .unwrap();
                let qty = rust_decimal::Decimal::from_str(&le.amount).unwrap();
                let obj = Ledger {
                    account_from_id: account_id_from_details((le).from_account.as_ref().unwrap()),
                    account_to_id: account_id_from_details((le).to_account.as_ref().unwrap()),
                    quantity: qty,
                    type_: le.type_.unwrap(),
                    ledger_entry_time: le_time,
                    transfer_time: ts,
                    vega_time: ctx.vega_time,
                    tx_hash: ctx.tx_hash,
                };
                self.pending.push(obj);
            }
        }
        Ok(())
    }

    fn flush(&mut self, conn: &mut postgres::Client) {
        let copied = Ledger::copy_in(&self.pending, conn).unwrap();
        assert!(copied == self.pending.len() as u64);
        self.pending.clear();
    }
}

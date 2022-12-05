use crate::entities::Ledger;
use crate::event_handlers::EventHandler;
use crate::event_handlers::InsertContext;
use crate::pending::{Batcher, Pending};
use crate::protos::events;
use crate::utils::account_id_from_details;
use anyhow::Result;
use std::str::FromStr;

impl EventHandler for events::LedgerMovements {
    fn handle(&self, ctx: &InsertContext, pending: &mut Pending) -> Result<()> {
        for lm in self.ledger_movements.iter() {
            for le in &lm.entries {
                let ts =
                    std::time::UNIX_EPOCH + std::time::Duration::from_nanos(le.timestamp as u64);
                let le_time = ctx.vega_time
                    + time::Duration::MICROSECOND
                        .checked_mul(pending.ledger_entries.len().try_into().unwrap())
                        .unwrap();
                let qty = rust_decimal::Decimal::from_str(&le.amount)?;
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
                pending.ledger_entries.add(obj);
            }
        }
        Ok(())
    }
}

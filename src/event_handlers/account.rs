use crate::entities::balance::Balances;
use crate::event_handlers::EventHandler;
use crate::event_handlers::InsertContext;
use crate::pending::{Batcher, Pending};
use crate::protos::vega;
use crate::utils::account_id;

use anyhow::{Context, Result};
use rust_decimal::Decimal;
use std::str::FromStr;

impl EventHandler for vega::Account {
    fn handle(&self, ctx: &InsertContext, pending: &mut Pending) -> Result<()> {
        let bal = Balances {
            account_id: account_id(&self.asset, &self.owner, &self.market_id, self.type_),
            balance: Decimal::from_str(&self.balance).context("parsing account balance")?,
            vega_time: ctx.vega_time,
            tx_hash: ctx.tx_hash,
        };
        pending.balances.add(bal);
        Ok(())
    }
}

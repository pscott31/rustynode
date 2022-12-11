use crate::entities::{Accounts, Balances, HexID};
use crate::event_handlers::EventHandler;
use crate::event_handlers::InsertContext;
use crate::pending::{Batcher, Pending};
use crate::protos::vega;
use crate::utils::account_id;
use anyhow::anyhow;

use anyhow::{Context, Result};
use rust_decimal::Decimal;
use std::str::FromStr;

impl EventHandler for vega::Account {
    fn handle(&self, ctx: &InsertContext, pending: &mut Pending) -> Result<()> {
        let aid = account_id(&self.asset, &self.owner, &self.market_id, self.type_);
        let bal = Balances {
            account_id: aid,
            balance: Decimal::from_str(&self.balance).context("parsing account balance")?,
            vega_time: ctx.vega_time,
            tx_hash: ctx.tx_hash,
        };
        pending.balances.add(bal);

        let party_id = if self.owner == "*" {
            "network"
        } else {
            &self.owner
        };

        let market_id = if self.market_id == "!" {
            ""
        } else {
            &self.market_id
        };

        let acc = Accounts {
            id: aid,
            party_id: HexID::try_from(party_id).context("parsing party id")?,
            asset_id: HexID::try_from(&self.asset).context("parsing asset id")?,
            market_id: HexID::try_from(market_id).context("parsing market id")?,
            type_: self.type_.value(),
            // .map_err(|x| anyhow!("unknown account type {x}"))?,
            vega_time: ctx.vega_time,
            tx_hash: ctx.tx_hash,
        };
        pending.accounts.add(acc);
        Ok(())
    }
}

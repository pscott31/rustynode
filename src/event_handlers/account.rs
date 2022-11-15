use crate::event_handlers::EventHandler;
use crate::hex_id::HexID;
use crate::protos::events;
use crate::utils::account_id;
use anyhow::{anyhow, Context, Result};
use postgres_macros::{PgCopyIn, PgTypes};
use postgres_macros_derive::{PgCopyIn, PgTypes};
use rust_decimal::Decimal;
use std::str::FromStr;
use std::{collections::HashMap, time::SystemTime};

#[derive(Debug, PgTypes, PgCopyIn)]
struct Balances {
    account_id: HexID,
    balance: rust_decimal::Decimal,
    vega_time: SystemTime,
    tx_hash: HexID,
}

impl Balances {
    fn key(&self) -> BalanceKey {
        BalanceKey {
            account_id: self.account_id,
            vega_time: self.vega_time,
        }
    }
}

#[derive(Eq, PartialEq, Hash)]
struct BalanceKey {
    account_id: HexID,
    vega_time: SystemTime,
}

pub struct AccountEventHandler {
    pending: HashMap<BalanceKey, Balances>,
}

impl AccountEventHandler {
    pub fn new() -> AccountEventHandler {
        return AccountEventHandler {
            pending: HashMap::new(),
        };
    }
}

impl EventHandler for AccountEventHandler {
    fn init(&mut self, _conn: &mut postgres::Client) {}
    fn handle(
        &mut self,
        ctx: &crate::event_handlers::InsertContext,
        _conn: &mut postgres::Client,
        e: &events::bus_event::Event,
    ) -> Result<()> {
        let events::bus_event::Event::Account(ae) = e else {
            return Ok(())
        };

        let bal = Balances {
            account_id: account_id(&ae.asset, &ae.owner, &ae.market_id, ae.type_),
            balance: Decimal::from_str(&ae.balance).context("parsing account balance")?,
            vega_time: ctx.vega_time,
            tx_hash: ctx.tx_hash,
        };

        self.pending.insert(bal.key(), bal);

        return Ok(());
    }

    fn flush(&mut self, conn: &mut postgres::Client) -> Result<()> {
        let copied = Balances::copy_in(self.pending.values(), conn).context("error inserting balances")?;
        assert!(copied == self.pending.len() as u64);
        if copied != self.pending.len() as u64 {
            return Err(anyhow!(
                "expected to copy {}, actually coped {} rows into database",
                self.pending.len(),
                copied
            ));
        }
        self.pending.clear();
        Ok(())
    }
}

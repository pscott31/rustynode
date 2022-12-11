use super::hex_id::HexID;
use crate::pending::HasKey;
use postgres_macros::{PgCopyIn, PgTypes};
use postgres_macros_derive::{PgCopyIn, PgTypes};
use std::time::SystemTime;

// Todo - postgres name for table so we can call this 'Balance'
#[derive(Debug, PgTypes, PgCopyIn)]
pub struct Balances {
    pub account_id: HexID,
    pub balance: rust_decimal::Decimal,
    pub vega_time: SystemTime,
    pub tx_hash: HexID,
}

impl HasKey for Balances {
    type Key = BalanceKey;
    fn key(&self) -> BalanceKey {
        BalanceKey {
            account_id: self.account_id,
            vega_time: self.vega_time,
        }
    }
}

#[derive(Eq, PartialEq, Hash)]
pub struct BalanceKey {
    account_id: HexID,
    vega_time: SystemTime,
}

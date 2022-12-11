use super::HexID;
use crate::pending::HasKey;
use postgres_macros::{PgCopyIn, PgTypes};
use postgres_macros_derive::{PgCopyIn, PgTypes};
use std::time::SystemTime;

#[derive(Debug, PgTypes, PgCopyIn)]
pub struct Accounts {
    pub id: HexID,
    pub party_id: HexID,
    pub asset_id: HexID,
    pub market_id: HexID,
    // pub type_: crate::protos::vega::AccountType,
    #[postgres = "type"]
    pub type_: i32,
    pub vega_time: SystemTime,
    pub tx_hash: HexID,
}

impl HasKey for Accounts {
    type Key = AccountKey;
    fn key(&self) -> AccountKey {
        AccountKey { id: self.id }
    }
}

#[derive(Eq, PartialEq, Hash)]
pub struct AccountKey {
    id: HexID,
}

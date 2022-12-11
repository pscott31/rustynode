use super::HexID;
use crate::pending::HasKey;
use postgres_macros::{PgCopyIn, PgTypes};
use postgres_macros_derive::{PgCopyIn, PgTypes};
use std::time::SystemTime;

#[derive(Debug, PgTypes, PgCopyIn)]
pub struct Orders {
    pub id: HexID,
    pub market_id: HexID,
    pub party_id: HexID,
    pub side: i16,
    pub price: rust_decimal::Decimal,
    pub size: i64,
    pub remaining: i64,
    pub time_in_force: i16,
    #[postgres = "type"]
    pub type_: i16,
    pub status: i16,
    pub reference: String,
    pub reason: i16,
    pub version: i32,
    pub batch_id: i32,
    pub pegged_offset: rust_decimal::Decimal,
    pub pegged_reference: i16,
    pub lp_id: HexID,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
    pub expires_at: SystemTime,
    pub tx_hash: HexID,
    pub vega_time: SystemTime,
    pub seq_num: i64,
    pub vega_time_to: SystemTime,
}

impl HasKey for Orders {
    type Key = OrderKey;
    fn key(&self) -> OrderKey {
        OrderKey {
            id: self.id,
            version: self.version,
        }
    }
}

#[derive(Eq, PartialEq, Hash)]
pub struct OrderKey {
    id: HexID,
    version: i32,
}

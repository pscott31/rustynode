use super::HexID;
use crate::protos::vega;
use postgres_macros::{PgCopyIn, PgTypes};
use postgres_macros_derive::{PgCopyIn, PgTypes};
use std::time::SystemTime;

#[derive(Debug, PgTypes, PgCopyIn)]
pub struct Ledger {
    pub account_from_id: HexID,
    pub account_to_id: HexID,
    pub quantity: rust_decimal::Decimal,
    #[postgres = "type"]
    pub type_: vega::TransferType,
    pub ledger_entry_time: SystemTime,
    pub transfer_time: SystemTime,
    pub vega_time: SystemTime,
    pub tx_hash: HexID,
}

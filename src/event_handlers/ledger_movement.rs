use crate::event_handlers::EventHandler;
use crate::hex_id::HexID;
use crate::protos::events;
use crate::protos::vega;
use crate::utils::account_id_from_details;
use postgres::binary_copy::BinaryCopyInWriter;
use postgres::types::ToSql;
use std::str::FromStr;
use std::sync::Once;
use std::time::SystemTime;

use postgres_macros::*;

//trait Badger<const N: usize> {
trait Badger {
    // fn hello_macro();
    fn types(conn: &mut postgres::Client) -> &'static [postgres::types::Type];
    // fn _types<const N: usize>(conn: &mut postgres::Client) -> [postgres::types::Type; N];
}

#[derive(Badger)]
struct Ledger {
    account_from_id: HexID,
    account_to_id: HexID,
    quantity: rust_decimal::Decimal,
    type_: vega::TransferType,
    ledger_entry_time: SystemTime,
    transfer_time: SystemTime,
    vega_time: SystemTime,
    tx_hash: HexID,
}

//impl<const N: usize> Badger<N> for LedgerEntry {
// impl Badger for LedgerEntry {
//     fn types(conn: &mut postgres::Client) -> &'static [postgres::types::Type] {
//         static mut TYPES: Option<[postgres::types::Type; 8]> = None;
//         static INIT: Once = Once::new();

//         unsafe {
//             INIT.call_once(|| {
//                 let q = "SELECT * from ledger WHERE
//                 account_from_id=$1 AND
//                 account_to_id=$2 AND
//                 quantity=$3 AND
//                 type=$4 AND
//                 ledger_entry_time=$5 AND
//                 transfer_time=$6 AND
//                 vega_time=$7 AND
//                 tx_hash=$8;";
//                 let stmt = conn.prepare(q).unwrap();
//                 let params = stmt.params();
//                 let types = [
//                     params[0].clone(),
//                     params[1].clone(),
//                     params[2].clone(),
//                     params[3].clone(),
//                     params[4].clone(),
//                     params[5].clone(),
//                     params[6].clone(),
//                     params[7].clone(),
//                 ];

//                 TYPES = Some(types);
//             });
//             TYPES.as_ref().unwrap()
//         }
//     }

// fn _types<const N: usize>(conn: &mut postgres::Client) -> [postgres::types::Type; N] {
//     let q = "SELECT * from ledger WHERE
//                 account_from_id=$1 AND
//                 account_to_id=$2 AND
//                 quantity=$3 AND
//                 type=$4 AND
//                 ledger_entry_time=$5 AND
//                 transfer_time=$6 AND
//                 vega_time=$7 AND
//                 tx_hash=$8;";
//     let stmt = conn.prepare(q).unwrap();
//     let params = stmt.params();
//     let types = [
//         params[0].clone(),
//         params[1].clone(),
//         params[2].clone(),
//         params[3].clone(),
//         params[4].clone(),
//         params[5].clone(),
//         params[6].clone(),
//         params[7].clone(),
//     ];
//     types
//  }
//}

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
        let types = Ledger::types(conn);

        let writer = conn
            .copy_in(
                "
                COPY ledger( account_from_id, account_to_id, quantity, type, ledger_entry_time,
                            transfer_time, vega_time, tx_hash) FROM STDIN (FORMAT binary)",
            )
            .unwrap();
        // TODO: get these types from PREPARE?
        // let types = [
        //     postgres::types::Type::BYTEA,
        //     postgres::types::Type::BYTEA,
        //     postgres::types::Type::NUMERIC,
        //     self.pg_type.clone(),
        //     postgres::types::Type::TIMESTAMPTZ,
        //     postgres::types::Type::TIMESTAMPTZ,
        //     postgres::types::Type::TIMESTAMPTZ,
        //     postgres::types::Type::BYTEA,
        // ];

        let mut writer = BinaryCopyInWriter::new(writer, types);

        for le in self.pending.iter() {
            let row: [&(dyn ToSql + Sync); 8] = [
                &le.account_from_id,
                &le.account_to_id,
                &le.quantity,
                &le.type_,
                &le.ledger_entry_time,
                &le.transfer_time,
                &le.vega_time,
                &le.tx_hash,
            ];
            writer.write(&row).unwrap()
        }
        let _copied = writer.finish().unwrap();
        self.pending.clear();
    }
}

// fn generate_le_type() -> postgres::types::Type {
//     let vals = vec![
//         String::from("TRANSFER_TYPE_UNSPECIFIED"),
//         String::from("TRANSFER_TYPE_LOSS"),
//         String::from("TRANSFER_TYPE_WIN"),
//         String::from("TRANSFER_TYPE_CLOSE"),
//         String::from("TRANSFER_TYPE_MTM_LOSS"),
//         String::from("TRANSFER_TYPE_MTM_WIN"),
//         String::from("TRANSFER_TYPE_MARGIN_LOW"),
//         String::from("TRANSFER_TYPE_MARGIN_HIGH"),
//         String::from("TRANSFER_TYPE_MARGIN_CONFISCATED"),
//         String::from("TRANSFER_TYPE_MAKER_FEE_PAY"),
//         String::from("TRANSFER_TYPE_MAKER_FEE_RECEIVE"),
//         String::from("TRANSFER_TYPE_INFRASTRUCTURE_FEE_PAY"),
//         String::from("TRANSFER_TYPE_INFRASTRUCTURE_FEE_DISTRIBUTE"),
//         String::from("TRANSFER_TYPE_LIQUIDITY_FEE_PAY"),
//         String::from("TRANSFER_TYPE_LIQUIDITY_FEE_DISTRIBUTE"),
//         String::from("TRANSFER_TYPE_BOND_LOW"),
//         String::from("TRANSFER_TYPE_BOND_HIGH"),
//         String::from("TRANSFER_TYPE_WITHDRAW_LOCK"),
//         String::from("TRANSFER_TYPE_WITHDRAW"),
//         String::from("TRANSFER_TYPE_DEPOSIT"),
//         String::from("TRANSFER_TYPE_BOND_SLASHING"),
//         String::from("TRANSFER_TYPE_STAKE_REWARD"),
//         String::from("TRANSFER_TYPE_TRANSFER_FUNDS_SEND"),
//         String::from("TRANSFER_TYPE_TRANSFER_FUNDS_DISTRIBUTE"),
//         String::from("TRANSFER_TYPE_CLEAR_ACCOUNT"),
//         String::from("TRANSFER_TYPE_CHECKPOINT_BALANCE_RESTORE"),
//     ];
//     let kind = postgres::types::Kind::Enum(vals);
//     postgres::types::Type::new(
//         String::from("transfer_type"),
//         0,
//         kind,
//         String::from("public"),
//     )
// }

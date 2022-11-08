pub mod ledger_movement {
    use crate::event_handlers::EventHandler;
    use crate::hex_id::HexID;
    use crate::protos::events;
    use crate::protos::vega;
    use crate::utils::account_id_from_details;
    use postgres::binary_copy::BinaryCopyInWriter;
    use postgres::types::ToSql;
    use std::str::FromStr;
    use std::time::SystemTime;
    use postgres_macros::*;
    trait PgTypes {
        fn types(conn: &mut postgres::Client) -> &'static [postgres::types::Type];
    }
    trait PgCopyIn {
        type Dave;
        fn copy_in<I>(items: I, conn: &mut postgres::Client) -> Result<u64, postgres::Error>
        where
            I: IntoIterator<Item = Self::Dave>;
    }
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
    #[automatically_derived]
    impl ::core::fmt::Debug for Ledger {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "account_from_id",
                "account_to_id",
                "quantity",
                "type_",
                "ledger_entry_time",
                "transfer_time",
                "vega_time",
                "tx_hash",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &&self.account_from_id,
                &&self.account_to_id,
                &&self.quantity,
                &&self.type_,
                &&self.ledger_entry_time,
                &&self.transfer_time,
                &&self.vega_time,
                &&self.tx_hash,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(f, "Ledger", names, values)
        }
    }
    impl PgTypes for Ledger {
        fn types(conn: &mut postgres::Client) -> &'static [postgres::types::Type] {
            static mut TYPES: Option<[postgres::types::Type; 8usize]> = None;
            static INIT: std::sync::Once = std::sync::Once::new();
            unsafe {
                INIT . call_once (| | { let q = { let res = :: alloc :: fmt :: format (:: core :: fmt :: Arguments :: new_v1 (& ["SELECT * from " , " WHERE "] , & [:: core :: fmt :: ArgumentV1 :: new_display (& "Ledger") , :: core :: fmt :: ArgumentV1 :: new_display (& "account_from_id=$1 AND account_to_id=$2 AND quantity=$3 AND type=$4 AND ledger_entry_time=$5 AND transfer_time=$6 AND vega_time=$7 AND tx_hash=$8")])) ; res } ; let stmt = conn . prepare (q . as_str ()) . unwrap () ; let params = stmt . params () ; let types = [params [0usize] . clone () , params [1usize] . clone () , params [2usize] . clone () , params [3usize] . clone () , params [4usize] . clone () , params [5usize] . clone () , params [6usize] . clone () , params [7usize] . clone ()] ; TYPES = Some (types) ; }) ;
                TYPES.as_ref().unwrap()
            }
        }
    }
    impl PgCopyIn for Ledger {
        type Dave = Ledger;
        fn copy_in<I>(items: I, conn: &mut postgres::Client) -> Result<u64, postgres::Error>
        where
            I: IntoIterator<Item = Self::Dave>,
        {
            let types = Ledger::types(conn);
            let q = {
                let res = :: alloc :: fmt :: format (:: core :: fmt :: Arguments :: new_v1 (& ["COPY " , "(" , ") FROM STDIN (FORMAT binary)"] , & [:: core :: fmt :: ArgumentV1 :: new_display (& "Ledger") , :: core :: fmt :: ArgumentV1 :: new_display (& "account_from_id,account_to_id,quantity,type,ledger_entry_time,transfer_time,vega_time,tx_hash")])) ;
                res
            };
            let writer = conn.copy_in(q.to_string()).unwrap();
            let mut writer = postgres::binary_copy::BinaryCopyInWriter::new(writer, types);
            for item in items {
                let row: [&(dyn ToSql + Sync); 8] = [
                    &item.account_from_id,
                    &item.account_to_id,
                    &item.quantity,
                    &item.type_,
                    &item.ledger_entry_time,
                    &item.transfer_time,
                    &item.vega_time,
                    &item.tx_hash,
                ];
                writer.write(&row).unwrap()
            }
            writer.finish()
        }
    }
    pub struct LedgerEventHandler {
        pending: Vec<Ledger>,
    }
    impl LedgerEventHandler {
        pub fn new() -> LedgerEventHandler {
            return LedgerEventHandler {
                pending: ::alloc::vec::Vec::new(),
            };
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
            let events :: bus_event :: Event :: LedgerMovements (e) = e else { return Ok (()) } ;
            for lm in e.ledger_movements.iter() {
                for le in &lm.entries {
                    let ts = std::time::UNIX_EPOCH
                        + std::time::Duration::from_nanos(le.timestamp as u64);
                    let le_time = ctx.vega_time
                        + time::Duration::MICROSECOND
                            .checked_mul(self.pending.len().try_into().unwrap())
                            .unwrap();
                    let qty = rust_decimal::Decimal::from_str(&le.amount).unwrap();
                    let obj = Ledger {
                        account_from_id: account_id_from_details(
                            (le).from_account.as_ref().unwrap(),
                        ),
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
}

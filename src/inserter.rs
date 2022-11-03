use std::str::FromStr;
use std::time::SystemTime;

use crate::hex_id::HexID;
use crate::protos::events;
use crate::protos::vega;
use postgres::binary_copy::BinaryCopyInWriter;
use postgres::types::ToSql;
use sha2::{Digest, Sha256};

trait EventHandler {
    fn handle_bus_event(&mut self, be: events::BusEvent) -> std::io::Result<()>;
}

fn sequence_number(be: &events::BusEvent) -> i32 {
    let (_block_height, seq) = be.id.split_once('-').unwrap();
    return seq.parse().unwrap();
}

pub struct Inserter<'a> {
    conn: postgres::Client,
    tx_hash: HexID,
    vega_time: std::time::SystemTime,
    syn_time: std::time::SystemTime,
    le_time: std::time::SystemTime,
    seq: i32,
    ledger_entries: Vec<LedgerEntry>,
    handlers: Vec<&'a dyn EventHandler>,
}

const NO_MARKET: &str = "!";
const SYSTEM_OWNER: &str = "*";

struct LedgerEntry {
    account_from_id: HexID,
    account_to_id: HexID,
    quantity: rust_decimal::Decimal,
    type_: vega::TransferType,
    ledger_entry_time: SystemTime,
    transfer_time: SystemTime,
    vega_time: SystemTime,
    tx_hash: HexID,
}

impl<'a> Inserter<'a> {
    pub fn new(conn: postgres::Client, handlers: Vec<&'a dyn EventHandler>) -> Inserter<'a> {
        return Inserter {
            conn: conn,
            tx_hash: HexID::from(""),
            vega_time: std::time::UNIX_EPOCH,
            le_time: std::time::UNIX_EPOCH,
            syn_time: std::time::UNIX_EPOCH,
            seq: 0,
            ledger_entries: vec![],
            handlers: handlers,
        };
    }

    pub fn handle_bus_event(&mut self, be: events::BusEvent) -> std::io::Result<()> {
        self.seq = sequence_number(&be);
        self.syn_time = self.vega_time + time::Duration::MICROSECOND * self.seq;
        self.tx_hash = HexID::from(be.tx_hash);
        match be.event {
            Option::Some(events::bus_event::Event::TimeUpdate(e)) => self.handle_time_update(e),
            Option::Some(events::bus_event::Event::LedgerMovements(e)) => {
                self.handle_ledger_movements(e)
            }
            Option::None => Ok(()),
            _ => Ok(()),
        }
    }

    fn handle_time_update(&mut self, e: events::TimeUpdate) -> std::io::Result<()> {
        self.vega_time =
            std::time::UNIX_EPOCH + std::time::Duration::from_nanos(e.timestamp as u64);
        self.le_time = self.vega_time;
        self.write_ledger_entries();
        Ok(())
    }

    fn write_ledger_entries(&mut self) {
        let vals = vec![
            String::from("TRANSFER_TYPE_UNSPECIFIED"),
            String::from("TRANSFER_TYPE_LOSS"),
            String::from("TRANSFER_TYPE_WIN"),
            String::from("TRANSFER_TYPE_CLOSE"),
            String::from("TRANSFER_TYPE_MTM_LOSS"),
            String::from("TRANSFER_TYPE_MTM_WIN"),
            String::from("TRANSFER_TYPE_MARGIN_LOW"),
            String::from("TRANSFER_TYPE_MARGIN_HIGH"),
            String::from("TRANSFER_TYPE_MARGIN_CONFISCATED"),
            String::from("TRANSFER_TYPE_MAKER_FEE_PAY"),
            String::from("TRANSFER_TYPE_MAKER_FEE_RECEIVE"),
            String::from("TRANSFER_TYPE_INFRASTRUCTURE_FEE_PAY"),
            String::from("TRANSFER_TYPE_INFRASTRUCTURE_FEE_DISTRIBUTE"),
            String::from("TRANSFER_TYPE_LIQUIDITY_FEE_PAY"),
            String::from("TRANSFER_TYPE_LIQUIDITY_FEE_DISTRIBUTE"),
            String::from("TRANSFER_TYPE_BOND_LOW"),
            String::from("TRANSFER_TYPE_BOND_HIGH"),
            String::from("TRANSFER_TYPE_WITHDRAW_LOCK"),
            String::from("TRANSFER_TYPE_WITHDRAW"),
            String::from("TRANSFER_TYPE_DEPOSIT"),
            String::from("TRANSFER_TYPE_BOND_SLASHING"),
            String::from("TRANSFER_TYPE_STAKE_REWARD"),
            String::from("TRANSFER_TYPE_TRANSFER_FUNDS_SEND"),
            String::from("TRANSFER_TYPE_TRANSFER_FUNDS_DISTRIBUTE"),
            String::from("TRANSFER_TYPE_CLEAR_ACCOUNT"),
            String::from("TRANSFER_TYPE_CHECKPOINT_BALANCE_RESTORE"),
        ];
        let kind = postgres::types::Kind::Enum(vals);
        let mytype = postgres::types::Type::new(
            String::from("transfer_type"),
            0,
            kind,
            String::from("public"),
        );

        let writer = self
            .conn
            .copy_in(
                "COPY ledger(
            account_from_id,
            account_to_id,
            quantity,
            type,
            ledger_entry_time,
            transfer_time,
            vega_time,
            tx_hash) FROM STDIN (FORMAT binary)",
            )
            .unwrap();
        let types = [
            postgres::types::Type::BYTEA,
            postgres::types::Type::BYTEA,
            postgres::types::Type::NUMERIC,
            mytype,
            postgres::types::Type::TIMESTAMPTZ,
            postgres::types::Type::TIMESTAMPTZ,
            postgres::types::Type::TIMESTAMPTZ,
            postgres::types::Type::BYTEA,
        ];

        let mut bwriter = BinaryCopyInWriter::new(writer, &types);

        for le in self.ledger_entries.iter() {
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
            bwriter.write(&row).unwrap()
        }
        let _copied = bwriter.finish().unwrap();
        self.ledger_entries.clear();
    }

    fn handle_ledger_movements(&mut self, e: events::LedgerMovements) -> std::io::Result<()> {
        let query = "
        INSERT INTO ledger (ledger_entry_time, account_from_id, account_to_id, quantity,
                            tx_hash, vega_time, transfer_time, type)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8);";

        for lm in e.ledger_movements.iter() {
            for le in &lm.entries {
                let ts =
                    std::time::UNIX_EPOCH + std::time::Duration::from_nanos(le.timestamp as u64);
                let qty = rust_decimal::Decimal::from_str(&le.amount).unwrap();

                let obj = LedgerEntry {
                    account_from_id: self.get_account_id((&le).from_account.as_ref().unwrap()),
                    account_to_id: self.get_account_id((&le).to_account.as_ref().unwrap()),
                    quantity: qty,
                    type_: le.type_.unwrap(),
                    ledger_entry_time: self.le_time,
                    transfer_time: ts,
                    vega_time: self.vega_time,
                    tx_hash: self.tx_hash,
                };

                self.ledger_entries.push(obj);
                self.le_time = self.le_time + time::Duration::MICROSECOND;
            }
        }

        Ok(())
    }

    fn get_account_id(&self, ad: &vega::AccountDetails) -> HexID {
        let mut hasher = Sha256::new();

        let party_id = match ad.owner.as_ref() {
            Some(x) => x.as_str(),
            None => SYSTEM_OWNER,
        };

        let market_id = match ad.market_id.as_ref() {
            Some(x) => x.as_str(),
            None => NO_MARKET,
        };

        //.unwrap_or(emptyString);
        hasher.update(&ad.asset_id);
        hasher.update(party_id);
        hasher.update(market_id);
        hasher.update(ad.type_.value().to_string()); // TODO - maybe needs to be ENUM_STR to match
        hasher.update(format!("{:?}", ad.type_.unwrap()));
        let result = hasher.finalize();
        let result_arr: [u8; 32] = result.as_slice().try_into().unwrap();
        HexID::from(result_arr)
    }
}

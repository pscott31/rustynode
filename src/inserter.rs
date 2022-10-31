use crate::hex_id::HexID;
use crate::protos::events;
use crate::protos::vega;

use rust_decimal::prelude::*;
use sha2::{Digest, Sha256};
// use std::string::String;
use postgres::types::{ToSql};

fn sequence_number(be: &events::BusEvent) -> i32 {
    let (_block_height, seq) = be.id.split_once('-').unwrap();
    return seq.parse().unwrap();
}

pub struct Inserter {
    conn: postgres::Client,
    tx_hash: HexID,
    vega_time: std::time::SystemTime,
    syn_time: std::time::SystemTime,
    le_time: std::time::SystemTime,
    seq: i32,
    ledger_entries: Vec<Box<dyn ToSql>>,
}

const NO_MARKET: &str = "!";
const SYSTEM_OWNER: &str = "*";

impl Inserter {
    pub fn new(conn: postgres::Client) -> Inserter {
        return Inserter {
            conn: conn,
            tx_hash: HexID::from(""),
            vega_time: std::time::UNIX_EPOCH,
            le_time: std::time::UNIX_EPOCH,
            syn_time: std::time::UNIX_EPOCH,
            seq: 0,
            ledger_entries: vec![],
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
        // TODO: round to microseconds
        self.vega_time =
            std::time::UNIX_EPOCH + std::time::Duration::from_nanos(e.timestamp as u64);
        self.le_time = self.vega_time;
        Ok(())
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
   
                self.ledger_entries.push(Box::new(self.le_time.clone()));
                // self.ledger_entries.push(Box::new(&self.get_account_id((&le).from_account.as_ref().unwrap())));
                // self.ledger_entries.push(Box::new(&self.get_account_id((&le).to_account.as_ref().unwrap())));
                // self.ledger_entries.push(Box::new(&qty));
                // self.ledger_entries.push(Box::new(&self.tx_hash));
                // self.ledger_entries.push(Box::new(&self.vega_time));
                // self.ledger_entries.push(Box::new(&le.type_.unwrap()));

                // let _res = self
                //     .conn
                //     .execute(
                //         query,
                //         &[
                //             &self.le_time,
                //             &self.get_account_id((&le).from_account.as_ref().unwrap()),
                //             &self.get_account_id((&le).to_account.as_ref().unwrap()),
                //             &qty,
                //             &self.tx_hash,
                //             &self.vega_time,
                //             &ts,
                //             &le.type_.unwrap(),
                //         ],
                //     )
                //     .unwrap();
                self.le_time = self.le_time + time::Duration::MICROSECOND
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

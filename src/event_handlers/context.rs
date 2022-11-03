use crate::hex_id::HexID;
use crate::protos::events;
use crate::protos::events::bus_event::Event;

use std::{
    process,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub struct InsertContext {
    pub tx_hash: HexID,
    pub vega_time: SystemTime,
    pub syn_time: SystemTime,
    pub seq: i32,
    pub height: u64,
}

// TODO: would be nice if this was impossible to exist in an uninitialized state
impl InsertContext {
    pub fn new() -> InsertContext {
        InsertContext {
            tx_hash: HexID::from(""),
            vega_time: UNIX_EPOCH,
            syn_time: UNIX_EPOCH,
            seq: 0,
            height: 0,
        }
    }

    pub fn update_from_event(&mut self, be: &events::BusEvent) {
        (self.height, self.seq) = parse_id(be);
        self.syn_time = self.vega_time + time::Duration::MICROSECOND * self.seq;
        self.tx_hash = HexID::from(be.tx_hash.as_str());

        if let Some(Event::TimeUpdate(tu)) = be.event.as_ref() {
            if self.height % 10 == 0 {
                println!("inserting block {}", self.height);
            }

            self.vega_time = UNIX_EPOCH + Duration::from_nanos(tu.timestamp as u64);
            self.syn_time = self.vega_time;

            if self.height > 100 {
                println!("stopping");
                process::exit(0);
            }
        }
    }
}

fn parse_id(be: &events::BusEvent) -> (u64, i32) {
    let (height, seq) = be.id.split_once('-').unwrap();
    return (height.parse().unwrap(), seq.parse().unwrap());
}

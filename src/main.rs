mod protos;
mod inserter;
mod hex_id;
mod testy;
mod enums;
use protos::events;
use std::io::{BufReader, Result};
//  use std::io::{Error, ErrorKind};
use std::fs::File;
use std::io::Read;
use protobuf::Message;
use byteorder::{BigEndian, ByteOrder};


fn next_event<T: Read>(reader: &mut T) -> Result<events::BusEvent>{
    let mut size_arr = [0u8; 4];
    reader.read_exact(&mut size_arr)?;
    let size = BigEndian::read_u32(&size_arr);

    let mut msg_vec = vec![0u8; size.try_into().unwrap()];
    reader.read_exact(&mut msg_vec)?;
     
    let be = events::BusEvent::parse_from_bytes(&msg_vec)?;
    Ok(be)
}

mod embedded {
use refinery::embed_migrations;
    embed_migrations!();
}

fn delete_everything(conn: &mut postgres::Client) {
    conn.execute("TRUNCATE TABLE ledger;", &[]).unwrap();
}
fn main() -> Result<()> {
    let mut conn = postgres::Client::connect("postgresql://vega:vega@localhost", postgres::NoTls).unwrap();
    embedded::migrations::runner().run(&mut conn).unwrap();
    delete_everything(&mut conn);
    let mut inserter = inserter::Inserter::new(conn);


    let f = File::open("/Users/philipscott/Downloads/eventlog.evt")?;
    let mut reader = BufReader::new(f);
    while let Ok(be) = next_event(&mut reader) {
        match inserter.handle_bus_event(be){
            Ok(_) => continue,
            Err(e)=> return Err(e),
        }
    }

    Ok(())
}

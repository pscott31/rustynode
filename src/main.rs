mod entities;
mod event_handlers;
mod inserter;
mod pending;
mod protos;
mod utils;

use anyhow::{Context, Result};
use byteorder::{BigEndian, ByteOrder};
use protobuf::Message;
use protos::events;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;

fn next_event<T: Read>(reader: &mut T) -> Result<events::BusEvent> {
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

fn truncate_table(table_name: &str, conn: &mut postgres::Client) -> Result<()> {
    let q = format!("TRUNCATE TABLE {};", table_name);
    conn.execute(q.as_str(), &[])?;
    Ok(())
}
fn delete_everything(conn: &mut postgres::Client) -> Result<()> {
    let tables = vec!["balances", "ledger", "accounts", "orders"];
    for table in tables {
        truncate_table(table, conn).context(format!("unable to truncate {}", table))?;
    }
    Ok(())
}
fn main() -> Result<()> {
    let conn_str = "postgresql://vega:vega@localhost";
    let mut conn = postgres::Client::connect(conn_str, postgres::NoTls)
        .context(format!("connecting to db {conn_str}"))?;

    embedded::migrations::runner()
        .run(&mut conn)
        .context("unable to migrate database schema")?;

    delete_everything(&mut conn).context("unable to delete existing data")?;

    let mut inserter = inserter::Inserter::new(conn);

    // let f = File::open("/home/scotty/work/testnet-2022-10-20.evt")?;
    let f = File::open("/Users/philipscott/Downloads/eventlog.evt")?;
    let mut reader = BufReader::new(f);
    while let Ok(be) = next_event(&mut reader) {
        inserter
            .handle_bus_event(&be)
            .with_context(|| format!("error handling event {}", be))?
    }

    Ok(())
}

use crate::entities::{Balances, Ledger};
use anyhow::{anyhow, Context, Result};
use postgres_macros::PgCopyIn;
use std::collections::HashMap;
use std::hash::Hash;
pub trait Batcher {
    type Item;
    fn add(&mut self, i: Self::Item);
    fn flush(&mut self, conn: &mut postgres::Client) -> Result<()>;
    fn len(&self) -> usize;
}

pub trait HasKey {
    type Key: Eq + Hash;
    fn key(&self) -> Self::Key;
}

pub struct MapBatcher<V: HasKey> {
    data: HashMap<V::Key, V>,
}

impl<T: PgCopyIn + HasKey> Batcher for MapBatcher<T> {
    type Item = T;
    fn flush(&mut self, conn: &mut postgres::Client) -> Result<()> {
        let actual = T::copy_in(self.data.values(), conn)?;
        check_len(self.len(), actual)?;
        self.data.clear();
        Ok(())
    }
    fn add(&mut self, item: T) {
        self.data.insert(item.key(), item);
    }
    fn len(&self) -> usize {
        self.data.len()
    }
}

impl<T: HasKey> Default for MapBatcher<T> {
    fn default() -> MapBatcher<T> {
        return MapBatcher {
            data: HashMap::new(),
        };
    }
}

/////////////////////// VecBatcher
pub struct VecBatcher<V> {
    data: Vec<V>,
}

impl<T: PgCopyIn> Batcher for VecBatcher<T> {
    type Item = T;
    fn flush(&mut self, conn: &mut postgres::Client) -> Result<()> {
        let actual = T::copy_in(&self.data, conn)?;
        check_len(self.len(), actual)?;
        self.data.clear();
        Ok(())
    }
    fn add(&mut self, item: Self::Item) {
        self.data.push(item);
    }
    fn len(&self) -> usize {
        return self.data.len();
    }
}

impl<T> Default for VecBatcher<T> {
    fn default() -> VecBatcher<T> {
        return VecBatcher { data: Vec::new() };
    }
}

#[derive(Default)]
pub struct Pending {
    pub balances: MapBatcher<Balances>,
    pub ledger_entries: VecBatcher<Ledger>,
}

impl Pending {
    pub fn flush(&mut self, conn: &mut postgres::Client) -> Result<()> {
        self.balances.flush(conn).context("flushing balances")?;
        self.ledger_entries.flush(conn).context("flushing ledger")?;
        Ok(())
    }
}

fn check_len(expected: usize, actual: u64) -> Result<()> {
    if actual != expected as u64 {
        return Err(anyhow!(
            "only {}/{} rows inserted into database",
            actual,
            expected
        ));
    }
    Ok(())
}

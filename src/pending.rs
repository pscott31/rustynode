use crate::entities::{Accounts, Balances, Ledger, Orders};
use anyhow::{anyhow, Context, Result};
use postgres_macros::PgCopyIn;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::mem;
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

/////////////////////// MapBatcher
/// TODO - make into an ordeed map like go version
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

/////////////////////// AddOnce
pub struct AddOnce<V: HasKey> {
    pending: HashMap<V::Key, V>,
    flushed: HashSet<V::Key>,
}

impl<T: PgCopyIn + HasKey> Batcher for AddOnce<T> {
    type Item = T;
    fn flush(&mut self, conn: &mut postgres::Client) -> Result<()> {
        let actual = T::copy_in(self.pending.values(), conn)?;
        check_len(self.len(), actual)?;
        let foo = mem::replace(&mut self.pending, HashMap::new());
        self.flushed.extend(foo.into_keys());
        Ok(())
    }
    fn add(&mut self, item: T) {
        let key = item.key();
        if !self.flushed.contains(&key) {
            self.pending.insert(key, item);
        }
    }
    fn len(&self) -> usize {
        self.pending.len()
    }
}

impl<T: HasKey> Default for AddOnce<T> {
    fn default() -> AddOnce<T> {
        return AddOnce {
            pending: HashMap::new(),
            flushed: HashSet::new(),
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

/////////////////////// Pending
#[derive(Default)]
pub struct Pending {
    pub balances: MapBatcher<Balances>,
    pub ledger_entries: VecBatcher<Ledger>,
    pub accounts: AddOnce<Accounts>,
    pub orders: MapBatcher<Orders>,
}

impl Pending {
    pub fn flush(&mut self, conn: &mut postgres::Client) -> Result<()> {
        self.balances.flush(conn).context("flushing balances")?;
        self.ledger_entries.flush(conn).context("flushing ledger")?;
        self.accounts.flush(conn).context("flushing accounts")?;
        self.orders.flush(conn).context("flushing orders")?;
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

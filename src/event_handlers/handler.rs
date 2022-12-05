use super::context::InsertContext;
use crate::pending::Pending;
use anyhow::Result;

pub trait EventHandler {
    fn handle(&self, ctx: &InsertContext, pending: &mut Pending) -> Result<()>;
}

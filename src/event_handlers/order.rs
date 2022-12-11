use crate::entities::{HexID, Orders};
use crate::event_handlers::EventHandler;
use crate::event_handlers::InsertContext;
use crate::pending::{Batcher, Pending};
use crate::protos::vega;
use anyhow::{Context, Result};
use rust_decimal::Decimal;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

struct NanoSeconds(i64);

impl From<NanoSeconds> for SystemTime {
    fn from(value: NanoSeconds) -> Self {
        UNIX_EPOCH + Duration::from_nanos(value.0 as u64)
    }
}

impl EventHandler for vega::Order {
    fn handle(&self, ctx: &InsertContext, pending: &mut Pending) -> Result<()> {
        // Would be nice to put nulls in the db if this isn't there but keep compatilbility with go version for now
        let (peg_offset, peg_ref) = match self.pegged_order.as_ref() {
            Some(peg) => {
                let o = Decimal::from_str(&peg.offset).context("parsing pegged offset")?;
                let r = i16::try_from(peg.reference.value()).context("parsing pegged reference")?;
                (o, r)
            }
            None => (Decimal::default(), 0),
        };

        let order = Orders {
            id: HexID::try_from(&self.id).context("parsing id")?,
            market_id: HexID::try_from(&self.market_id).context("parsing market id")?,
            party_id: HexID::try_from(&self.party_id).context("parsing party id")?,
            side: i16::try_from(self.side.value()).context("parsing side")?,
            price: rust_decimal::Decimal::from_str(&self.price).context("parsing order")?,
            size: i64::try_from(self.size).context("parsing size")?,
            remaining: i64::try_from(self.remaining).context("parsing remaining")?,
            time_in_force: i16::try_from(self.time_in_force.value())
                .context("parsing time in force")?,
            type_: i16::try_from(self.type_.value()).context("parsing type")?,
            status: i16::try_from(self.status.value()).context("parsing status")?,
            reference: self.reference.clone(),
            reason: i16::try_from(self.reason.value()).context("parsing reason")?,
            version: i32::try_from(self.version).context("parsing version")?,
            batch_id: i32::try_from(self.batch_id).context("parsing batch id")?,
            pegged_offset: peg_offset,
            pegged_reference: peg_ref,
            lp_id: HexID::try_from(&self.liquidity_provision_id).context("parsing lp id")?,
            created_at: NanoSeconds(self.created_at).into(),
            updated_at: NanoSeconds(self.updated_at).into(),
            expires_at: NanoSeconds(self.expires_at).into(),
            tx_hash: ctx.tx_hash,
            vega_time: ctx.vega_time,
            seq_num: ctx.seq.into(),
            vega_time_to: UNIX_EPOCH, // TODO.. optional?
        };
        pending.orders.add(order);
        Ok(())
    }
}

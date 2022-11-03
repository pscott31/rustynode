use sha2::{Digest, Sha256};

use crate::hex_id::HexID;
use crate::protos::vega;

const NO_MARKET: &str = "!";
const SYSTEM_OWNER: &str = "*";

pub fn account_id_from_details(ad: &vega::AccountDetails) -> HexID {
    let mut hash = Sha256::new();

    let party_id = match ad.owner.as_ref() {
        Some(x) => x.as_str(),
        None => SYSTEM_OWNER,
    };

    let market_id = match ad.market_id.as_ref() {
        Some(x) => x.as_str(),
        None => NO_MARKET,
    };

    hash.update(&ad.asset_id);
    hash.update(party_id);
    hash.update(market_id);
    hash.update(ad.type_.value().to_string()); // TODO - maybe needs to be ENUM_STR to match
    hash.update(format!("{:?}", ad.type_.unwrap()));
    let result = hash.finalize();
    let result_arr: [u8; 32] = result.as_slice().try_into().unwrap();
    HexID::from(result_arr)
}

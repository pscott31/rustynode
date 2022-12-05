use sha2::{Digest, Sha256};

use crate::entities::HexID;
use crate::protos::vega;

const NO_MARKET: &str = "!";
const SYSTEM_OWNER: &str = "*";

pub fn account_id_from_details(ad: &vega::AccountDetails) -> HexID {
    let party_id = match ad.owner.as_ref() {
        Some(x) => x.as_str(),
        None => SYSTEM_OWNER,
    };

    let market_id = match ad.market_id.as_ref() {
        Some(x) => x.as_str(),
        None => NO_MARKET,
    };

    account_id(&ad.asset_id, party_id, market_id, ad.type_)
}

pub fn account_id(
    asset_id: &str,
    party_id: &str,
    market_id: &str,
    type_: protobuf::EnumOrUnknown<vega::AccountType>,
) -> HexID {
    let mut hash = Sha256::new();
    hash.update(asset_id);
    hash.update(party_id);
    hash.update(market_id);
    hash.update(format!("{:?}", type_.unwrap()));
    let result = hash.finalize();
    let result_arr: [u8; 32] = result.as_slice().try_into().unwrap();
    HexID::from(result_arr)
}

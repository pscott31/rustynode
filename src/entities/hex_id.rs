use anyhow::Result;
use bytes::BytesMut;
use postgres::types::{to_sql_checked, IsNull, ToSql, Type};
use std::error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HexID {
    data: [u8; 32],
}

impl TryFrom<&String> for HexID {
    type Error = anyhow::Error;
    fn try_from(s: &String) -> Result<Self> {
        hex_from(s)
    }
}

impl TryFrom<&str> for HexID {
    type Error = anyhow::Error;
    fn try_from(s: &str) -> Result<Self> {
        hex_from(s)
    }
}

impl From<[u8; 32]> for HexID {
    fn from(d: [u8; 32]) -> Self {
        return HexID { data: d };
    }
}

fn hex_from(s: &str) -> Result<HexID> {
    // fn hex_from<T: AsRef<[u8]>>(s: T) -> Result<HexID> {

    let v = match s {
        "VOTE" => "00",
        "network" => "03",
        "XYZalpha" => "04",
        "XYZbeta" => "05",
        "XYZdelta" => "06",
        "XYZepsilon" => "07",
        "XYZgamma" => "08",
        "fBTC" => "09",
        "fDAI" => "0a",
        "fEURO" => "0b",
        "fUSDC" => "0c",
        _ => s,
    };

    let mut v = hex::decode(v)?;
    v.resize(32, 0);
    return Ok(HexID {
        data: v.try_into().unwrap(),
    });
}

impl ToSql for HexID {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        self.data.to_sql(ty, out)
    }
    fn accepts(ty: &Type) -> bool {
        <[u8; 32]>::accepts(ty)
    }
    to_sql_checked!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn internal() {
        let arse = HexID::try_from("ff").unwrap();
        let bandit: [u8; 32] = [
            255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        assert_eq!(arse.data, bandit);
    }
}

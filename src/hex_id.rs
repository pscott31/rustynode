use postgres::types::{ToSql, Type, IsNull, to_sql_checked};
use bytes::BytesMut;
use std::error::Error;

#[derive(Debug, Clone, Copy)]
pub struct HexID {
    data: [u8; 32]
}

impl From<String> for HexID {
    fn from(s: String) -> Self {
        hex_from(s)
    }
}

impl From<&str> for HexID {
    fn from(s: &str) -> Self {
        hex_from(s)
    }
}

impl From<[u8; 32]> for HexID {
    fn from(d: [u8; 32]) -> Self {
        return HexID{data: d}
    }
}
fn hex_from<T: AsRef<[u8]>>(s: T) -> HexID {
    let mut v = hex::decode(s).unwrap();
    v.resize(32, 0);
    return HexID{data: v.try_into().unwrap()}
}

impl ToSql for HexID {
    fn to_sql(&self, ty: &Type, out: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>>{
        self.data.to_sql(ty, out)
    }
    fn accepts(ty: &Type) -> bool{
        <[u8; 32]>::accepts(ty)
    }
    to_sql_checked!();
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn internal() {
        let arse = HexID::from("ff");
        let bandit: [u8;32] = [255,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];
        assert_eq!(arse.data, bandit );
    }
}

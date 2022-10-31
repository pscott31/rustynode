// use crate::hex_id::HexID;
// use crate::protos::events;
// use crate::protos::vega;

// use postgres::types::{to_sql_checked, ToSql}; //accepts
// use rust_decimal::prelude::*;
// use sha2::{Digest, Sha256};
// use std::string::String;


// macro_rules! enum_sql_impl{
//     ($ty:ty) => {
//         impl ToSql for $ty {
//             fn to_sql(
//                 &self,
//             _ty: &postgres_types::Type,
//                 buf: &mut bytes::BytesMut,
//             ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
//             wherea
//                 Self: Sized,
//             {
//                 let s = match *self {
//                     vega::TransferType::TRANSFER_TYPE_UNSPECIFIED => "TRANSFER_TYPE_UNSPECIFIED",
//                     vega::TransferType::TRANSFER_TYPE_LOSS => "TRANSFER_TYPE_LOSS",
//                     vega::TransferType::TRANSFER_TYPE_WIN => "TRANSFER_TYPE_WIN",
//                     _ => "TRANSFER_TYPE_UNSPECIFIED", //TODO
//                 };
//                 buf.extend_from_slice(s.as_bytes());
//                 std::result::Result::Ok(postgres_types::IsNull::No)
//             }

//     fn accepts(type_: &postgres_types::Type) -> bool {
//         if type_.name() != "transfer_type" {
//             return false;
//         }
//         match *type_.kind() {
//             ::postgres_types::Kind::Enum(ref variants) => {
//                 // TODO - put back size check?
//                 // if variants.len() != 2usize {
//                 //     return false;
//                 // }
//                 variants.iter().all(|v| match &**v {
//                     "TRANSFER_TYPE_UNSPECIFIED" => true,
//                     "TRANSFER_TYPE_LOSS" => true,
//                      _ => true, // TODO put back?
//                     // _ => false,
//                 })
//             }
//             _ => false,
//         }
//     }

//     to_sql_checked!();            
//         }
//     }
// }

// enum_sql_impl!(vega::TransferType);
// use postgres::types::Type;
// use std::collections::HashMap;

// // fn fetch_type(conn: &mut postgres::Client) -> Result<Type, postgres::Error> {}

// pub struct TypeCache {
//     types: HashMap<String, Type>,
// }

// impl TypeCache {
//     pub fn new() -> TypeCache {
//         TypeCache {
//             types: HashMap::new(),
//         }
//     }

//     // Get the postgres type information by preparing a trivial query
//     pub fn get(
//         &mut self,
//         name: &str,
//         conn: &mut postgres::Client,
//     ) -> Result<Type, postgres::Error> {
//         let stmt = conn.prepare(format!("SELECT $1::{name}").as_str())?;
//         let type_ = stmt.params()[0].clone();
//         return Ok(type_);
//     }
// }

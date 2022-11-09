pub trait PgTypes {
    fn types(conn: &mut postgres::Client) -> &'static [postgres::types::Type];
}

pub trait PgCopyIn {
    type Item;
    fn copy_in(items: &[Self::Item], conn: &mut postgres::Client) -> Result<u64, postgres::Error>;
}

pub trait PgTypes {
    fn types(conn: &mut postgres::Client) -> &'static [postgres::types::Type];
}

pub trait PgCopyIn {
    type Item<'a>;
    fn copy_in<'a, I>(items: I, conn: &mut postgres::Client) -> Result<u64, postgres::Error>
    where
        I: IntoIterator<Item = Self::Item<'a>>;
}

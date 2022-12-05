pub trait PgTypes {
    fn types(conn: &mut postgres::Client) -> &'static [postgres::types::Type];
}

pub trait PgCopyIn
where
    Self: 'static,
{
    fn copy_in<'a, I>(items: I, conn: &mut postgres::Client) -> Result<u64, postgres::Error>
    where
        I: IntoIterator<Item = &'a Self>;
}

// pub trait PgCopyIn
// where
//     Self: 'static,
// {
//     fn copy_in<'a, I>(items: I)
//     where
//         I: IntoIterator<Item = &'a Self>;
// }

use proc_macro::TokenStream;
use quote::quote;
use syn;

trait PgTypes {
    fn types(conn: &mut postgres::Client) -> &'static [postgres::types::Type];
}

trait PgCopyIn {
    type Item;
    fn copy_in(items: &[Self::Item], conn: &mut postgres::Client) -> Result<u64, postgres::Error>;
}

/////////////////////////////////////////////////////////// PgTypes

#[proc_macro_derive(PgTypes, attributes(postgres))]
pub fn pg_types_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_pg_types_macro(&ast)
}

fn impl_pg_types_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let column_names = pg_column_names(ast);
    let table_name = name.to_string();
    let n_cols = column_names.len();
    let col_iter = 0..(n_cols);

    let where_clause = column_names
        .iter()
        .enumerate()
        .map(|(i, field_name)| format!("{field_name}=${}", i + 1))
        .collect::<Vec<String>>()
        .join(" AND ");

    let gen = quote! {
        impl PgTypes for #name {
            fn types(conn: &mut postgres::Client) -> &'static [postgres::types::Type] {
                static mut TYPES: Option<[postgres::types::Type; #n_cols]> = None;
                static INIT: std::sync::Once = std::sync::Once::new();
                unsafe {
                    INIT.call_once(|| {
                        let q = format!("SELECT * from {} WHERE {}", #table_name, #where_clause);
                        let stmt = conn.prepare(q.as_str()).unwrap();
                        let params = stmt.params();
                        let types = [ #(params[#col_iter].clone(),)*];
                        TYPES = Some(types);
                    });
                    TYPES.as_ref().unwrap()
                }
            }

        }
    };
    gen.into()
}

/////////////////////////////////////////////////////////// PgCopyIn

#[proc_macro_derive(PgCopyIn, attributes(postgres))]
pub fn pg_types_copy_in(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_copy_in_macro(&ast)
}

fn impl_copy_in_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let column_names = pg_column_names(ast);
    let column_list = column_names.join(",");
    let num_columns = column_names.len();
    let table_name = name.to_string();

    let syn::Data::Struct(syn::DataStruct{fields: syn::Fields::Named(ref fields), ..}) = ast.data else {
        panic!("must derive from data struct")
    };

    let identifiers = fields.named.iter().map(|a| a.ident.as_ref().unwrap());

    let gen = quote! {
        use postgres::types::ToSql;
        impl PgCopyIn for #name {
            // type Item<'a> = & 'a #name;
            // fn copy_in<'a, I>(items: I)
            // where
            //     I: IntoIterator<Item = &'a Self>,
            fn copy_in<'a, I>(items: I, conn: &mut postgres::Client) -> Result<u64, postgres::Error>
                where I: IntoIterator<Item = &'a Self>,
            {
                let types = #name::types(conn);
                let q = format!("COPY {}({}) FROM STDIN (FORMAT binary)", #table_name, #column_list);
                let writer = conn.copy_in(&q.to_string()).unwrap();
                let mut writer = postgres::binary_copy::BinaryCopyInWriter::new(writer, types);

                for item in items {
                    let row: [&(dyn ToSql + Sync); #num_columns] = [#(&item.#identifiers),*];
                    writer.write(&row).unwrap()
                }
                writer.finish()
            }
        }
    };
    gen.into()
}

/////////////////////////////////////////////////////////// Helpers

fn pg_name(f: &syn::Field) -> String {
    for attr in f.attrs.iter() {
        let meta = attr.parse_meta().unwrap();
        if !meta.path().is_ident("postgres") {
            continue;
        }
        if let Ok(syn::Meta::NameValue(nv)) = attr.parse_meta() {
            if let syn::Lit::Str(x) = nv.lit {
                return x.value();
            }
        }
    }
    return f.ident.as_ref().unwrap().to_string();
}

fn pg_column_names(ast: &syn::DeriveInput) -> Vec<String> {
    match &ast.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(ref fields),
            ..
        }) => fields
            .named
            .iter()
            .map(|f| pg_name(f))
            .collect::<Vec<String>>(),
        _ => {
            panic!("can only use on named structs")
        }
    }
}

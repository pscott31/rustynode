use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_derive(Badger)]
pub fn hello_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_hello_macro(&ast)
}

fn impl_hello_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let field_names = match &ast.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(ref fields),
            ..
        }) => fields
            .named
            .iter()
            .map(|f| f.ident.as_ref().unwrap().to_string())
            .collect::<Vec<String>>(),
        _ => {
            panic!("can only use on structs")
        }
    };

    let table_name = name.to_string();
    let arse = field_names.iter().enumerate();
    let cake = arse.map(|(i, field_name)| format!("{field_name}=${}", i + 1));
    let dandy: Vec<String> = cake.collect();
    let n_cols = dandy.len();
    let col_iter = 0..(n_cols);
    let selecta = dandy.join(" AND ");

    let gen = quote! {
        impl Badger for #name {
            fn types(conn: &mut postgres::Client) -> &'static [postgres::types::Type] {
                static mut TYPES: Option<[postgres::types::Type; #n_cols]> = None;
                static INIT: Once = Once::new();
                unsafe {
                    INIT.call_once(|| {
                        let q = format!("SELECT * from {} WHERE {}", #table_name, #selecta);
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

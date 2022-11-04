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
    // println!("Wrapping {name}");

    let gen = quote! {
        impl Badger for #name {
            fn types(conn: &mut postgres::Client) -> &'static [postgres::types::Type] {
                // println!("badger time: {}", #name)
                static mut TYPES: Option<[postgres::types::Type; 8]> = None;
                static INIT: Once = Once::new();
                unsafe {
                    INIT.call_once(|| {
                        let q = "SELECT * from ledger WHERE
                        account_from_id=$1 AND
                        account_to_id=$2 AND
                        quantity=$3 AND
                        type=$4 AND
                        ledger_entry_time=$5 AND
                        transfer_time=$6 AND
                        vega_time=$7 AND
                        tx_hash=$8;";
                        let stmt = conn.prepare(q).unwrap();
                        let params = stmt.params();
                        let types = [
                            params[0].clone(),
                            params[1].clone(),
                            params[2].clone(),
                            params[3].clone(),
                            params[4].clone(),
                            params[5].clone(),
                            params[6].clone(),
                            params[7].clone(),
                        ];

                        TYPES = Some(types);
                    });
                    TYPES.as_ref().unwrap()
                }
            }

        }
    };
    gen.into()
}

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Structly, attributes(structly, structly_if))]
pub fn derive_structly(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(named) => &named.named,
            _ => panic!("only named fields are supported"),
        },
        _ => panic!("only structs are supported"),
    };
    eprintln!("{:#?}", input);

    quote! {
        impl ::structly::Verify for #struct_name {
            fn verify(&self) -> Result<(), Vec<String>> {
                let errors: Vec<String> = Vec::new();
                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(errors)
                }
            }
        }
    }.into()
}
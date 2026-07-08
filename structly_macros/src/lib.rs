use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Structly, attributes(structly, structly_if))]
pub fn derive_structly(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    quote! {
        impl #struct_name {
            pub fn validate(&self) -> Result<(), Vec<String>> {
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
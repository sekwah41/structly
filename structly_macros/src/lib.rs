use proc_macro::TokenStream;
use quote::quote;
use structly_core::{parse_field_config, FieldConfig};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// Generate a field's `verify()` checks (a rule fails when `when` holds but the
/// field is `None`); `mode` selects `all` / `fail_fast` / `any` combining.
fn field_checks(field: &syn::Field, config: &FieldConfig) -> proc_macro2::TokenStream {
    if config.rules.is_empty() {
        return proc_macro2::TokenStream::new();
    }

    let field_ident = field.ident.as_ref().unwrap();
    let field_str = field_ident.to_string();
    let mode = config.mode.as_deref().unwrap_or("all");

    let push = |reason: &str| {
        quote! {
            errors.push(::structly::ValidationError::new(#field_str, #reason));
        }
    };

    match mode {
        "fail_fast" => {
            // An `if / else if` chain naturally reports only the first match.
            let mut chain = proc_macro2::TokenStream::new();
            for (i, rule) in config.rules.iter().enumerate() {
                let cond = &rule.condition;
                let push = push(&rule.reason);
                let branch = quote! {
                    if (#cond) && self.#field_ident.is_none() {
                        #push
                    }
                };
                if i == 0 {
                    chain.extend(branch);
                } else {
                    chain.extend(quote! { else #branch });
                }
            }
            chain
        }
        "any" => {
            let passes = config.rules.iter().map(|rule| {
                let cond = &rule.condition;
                quote! { !((#cond) && self.#field_ident.is_none()) }
            });
            // Every rule failed: collapse them into one human-readable error
            // listing the alternatives, built at expansion time.
            let mut combined = String::from("One of the following must be true:");
            for rule in &config.rules {
                combined.push_str("\n - ");
                combined.push_str(&rule.reason);
            }
            let push = push(&combined);
            quote! {
                if !( false #( || #passes )* ) {
                    #push
                }
            }
        }
        // "all" and the default.
        _ => {
            let checks = config.rules.iter().map(|rule| {
                let cond = &rule.condition;
                let push = push(&rule.reason);
                quote! {
                    if (#cond) && self.#field_ident.is_none() {
                        #push
                    }
                }
            });
            quote! { #(#checks)* }
        }
    }
}

fn expand(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name = &input.ident;

    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(named) => &named.named,
            _ => return Err(syn::Error::new_spanned(struct_name, "only named fields are supported")),
        },
        _ => return Err(syn::Error::new_spanned(struct_name, "only structs are supported")),
    };

    let mut checks = Vec::new();
    for field in fields {
        let config = parse_field_config(field)?;
        let mut per_field = field_checks(field, &config);

        // `#[structly(nested)]`: recurse into the field's own `verify()` and
        // prefix each error's path with this field's name (`section.field`).
        if config.nested {
            let field_ident = field.ident.as_ref().unwrap();
            let field_str = field_ident.to_string();
            per_field.extend(quote! {
                if let Err(nested_errors) = ::structly::Verify::verify(&self.#field_ident) {
                    for mut nested_error in nested_errors {
                        nested_error.field = ::std::format!("{}.{}", #field_str, nested_error.field);
                        errors.push(nested_error);
                    }
                }
            });
        }

        if !per_field.is_empty() {
            checks.push(per_field);
        }
    }

    let body = if checks.is_empty() {
        quote! { Ok(()) }
    } else {
        quote! {
            let mut errors: Vec<::structly::ValidationError> = Vec::new();
            #(#checks)*
            if errors.is_empty() {
                Ok(())
            } else {
                Err(errors)
            }
        }
    };

    Ok(quote! {
        impl ::structly::Verify for #struct_name {
            fn verify(&self) -> Result<(), Vec<::structly::ValidationError>> {
                #body
            }
        }
    })
}


#[proc_macro_derive(Structly, attributes(structly, structly_if))]
pub fn derive_structly(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match expand(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Expr, Fields, LitStr};
use syn::meta::ParseNestedMeta;

macro_rules! str_arg {
    ($meta:expr, $target:expr, $name:literal) => {{
        if $target.is_some() {
            return Err($meta.error(concat!("duplicate `", $name, "` argument")));
        }
        $target = Some($meta.value()?.parse::<LitStr>()?.value());
    }};
}

fn parse_enum_arg(
    meta: &ParseNestedMeta,
    allowed: &[&str],
    arg_name: &str,
) -> syn::Result<String> {
    let lit: LitStr = meta.value()?.parse()?;
    let value = lit.value();

    if allowed.contains(&value.as_str()) {
        Ok(value)
    } else {
        Err(syn::Error::new(
            lit.span(),
            format!(
                "unknown {arg_name} `{value}` - expected one of: {}",
                allowed.join(", ")
            ),
        ))
    }
}

struct Rule {
    condition: Expr,
    reason: String,
}

#[derive(Default)]
struct FieldConfig {
    name: Option<String>,
    description: Option<String>,
    mode: Option<String>,
    rules: Vec<Rule>,
}

/// Collect a field's `structly` / `structly_if` attributes into a [`FieldConfig`].
fn parse_field_config(field: &syn::Field) -> syn::Result<FieldConfig> {
    let mut config = FieldConfig::default();

    for attr in &field.attrs {
        if attr.path().is_ident("structly") {
            parse_structly_attr(attr, &mut config)?;
        } else if attr.path().is_ident("structly_if") {
            config.rules.push(parse_structly_if_attr(attr)?);
        }
    }

    Ok(config)
}

/// Parse a single `#[structly(...)]` attribute into `config`.
fn parse_structly_attr(attr: &syn::Attribute, config: &mut FieldConfig) -> syn::Result<()> {
    attr.parse_nested_meta(|meta| {
        match meta.path.get_ident().map(|i| i.to_string()).as_deref() {
            Some("name") => str_arg!(meta, config.name, "name"),
            Some("description") => str_arg!(meta, config.description, "description"),
            Some("mode") => {
                if config.mode.is_some() {
                    return Err(meta.error("duplicate `mode` argument"));
                }
                config.mode = Some(parse_enum_arg(&meta, &["all", "fail_fast", "any"], "mode")?);
            }
            Some(other) => {
                return Err(meta.error(format!(
                    "unknown structly argument `{other}` - expected `name`, `description`, or `mode`"
                )));
            }
            None => return Err(meta.error("expected a simple identifier")),
        }
        Ok(())
    })
}

/// Parse a single `#[structly_if(when = <expr>, reason = "<text>")]` attribute into a [`Rule`].
fn parse_structly_if_attr(attr: &syn::Attribute) -> syn::Result<Rule> {
    let mut condition: Option<Expr> = None;
    let mut reason: Option<String> = None;

    attr.parse_nested_meta(|meta| {
        match meta.path.get_ident().map(|i| i.to_string()).as_deref() {
            Some("when") => {
                if condition.is_some() {
                    return Err(meta.error("duplicate `when` argument"));
                }
                condition = Some(meta.value()?.parse()?);
            }
            Some("reason") => str_arg!(meta, reason, "reason"),
            Some(other) => {
                return Err(meta.error(format!(
                    "unknown structly_if argument `{other}` - expected `when` or `reason`"
                )));
            }
            None => return Err(meta.error("expected a simple identifier")),
        }
        Ok(())
    })?;

    let condition =
        condition.ok_or_else(|| syn::Error::new_spanned(attr, "`structly_if` requires a `when` condition"))?;
    let reason =
        reason.ok_or_else(|| syn::Error::new_spanned(attr, "`structly_if` requires a `reason`"))?;

    Ok(Rule { condition, reason })
}

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
            errors.push(::structly::ValidationError { field: #field_str, reason: #reason });
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
        let field_checks = field_checks(field, &config);
        if !field_checks.is_empty() {
            checks.push(field_checks);
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
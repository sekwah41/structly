//! Shared syn-level attribute parsing, documentation model, and markdown
//! rendering used by both the `structly_macros` derive and the `structly` CLI.

use quote::ToTokens;
use syn::meta::ParseNestedMeta;
use syn::{Expr, ExprLit, Lit, LitStr};

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

pub struct Rule {
    pub condition: Expr,
    pub reason: String,
}

#[derive(Default)]
pub struct FieldConfig {
    pub name: Option<String>,
    pub description: Option<String>,
    pub mode: Option<String>,
    pub rules: Vec<Rule>,
    /// `#[structly(nested)]`: recurse into this field's own `verify()`.
    pub nested: bool,
}

/// Collect a field's `structly` / `structly_if` attributes into a [`FieldConfig`].
///
/// When no `#[structly(description = ...)]` is given, the field's `///` doc
/// comment (if any) is used as the description, like clap does for help text.
pub fn parse_field_config(field: &syn::Field) -> syn::Result<FieldConfig> {
    let mut config = FieldConfig::default();

    for attr in &field.attrs {
        if attr.path().is_ident("structly") {
            parse_structly_attr(attr, &mut config)?;
        } else if attr.path().is_ident("structly_if") {
            config.rules.push(parse_structly_if_attr(attr)?);
        }
    }

    if config.description.is_none() {
        config.description = doc_comment(&field.attrs);
    }

    Ok(config)
}

/// Join a field's `///` lines (each a `#[doc = "..."]` attribute) into one
/// multiline description, dropping only the single leading space rustdoc
/// leaves on each line (`/// text` is stored as `" text"`).
fn doc_comment(attrs: &[syn::Attribute]) -> Option<String> {
    let mut lines: Vec<String> = Vec::new();

    for attr in attrs {
        if !attr.path().is_ident("doc") {
            continue;
        }
        if let syn::Meta::NameValue(nv) = &attr.meta {
            if let Expr::Lit(ExprLit { lit: Lit::Str(text), .. }) = &nv.value {
                let line = text.value();
                lines.push(line.strip_prefix(' ').unwrap_or(&line).to_string());
            }
        }
    }

    (!lines.is_empty()).then(|| lines.join("\n"))
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
            Some("nested") => config.nested = true,
            Some(other) => {
                return Err(meta.error(format!(
                    "unknown structly argument `{other}` - expected `name`, `description`, `mode`, or `nested`"
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

// ---------------------------------------------------------------------------
// Documentation model
// ---------------------------------------------------------------------------

/// One `structly_if` rule, with its condition rendered back to source text.
#[derive(Debug, Clone)]
pub struct RuleDoc {
    pub condition: String,
    pub reason: String,
}

/// Documentation for one field of a `#[derive(Structly)]` struct.
#[derive(Debug, Clone)]
pub struct FieldDoc {
    /// The field's identifier.
    pub field: String,
    /// Human-readable name (`#[structly(name = ...)]`, defaults to the identifier).
    pub name: String,
    pub description: String,
    /// Rule-combining mode: `all` / `fail_fast` / `any`.
    pub mode: String,
    pub rules: Vec<RuleDoc>,
    /// Whether the field is marked `#[structly(nested)]`.
    pub nested: bool,
    /// The nested struct's own documentation, when it could be resolved.
    pub section: Option<StructDoc>,
}

/// Documentation for a whole `#[derive(Structly)]` struct.
#[derive(Debug, Clone)]
pub struct StructDoc {
    pub name: String,
    pub fields: Vec<FieldDoc>,
}

/// Render a condition expression back to source text, preferring the original
/// spelling (`Span::source_text`) and falling back to the token stream.
pub fn condition_source(expr: &Expr) -> String {
    use syn::spanned::Spanned;
    expr.span()
        .source_text()
        .unwrap_or_else(|| tidy_token_string(&expr.to_token_stream().to_string()))
}

/// Best-effort cleanup of `TokenStream::to_string` spacing (`self . a` -> `self.a`).
fn tidy_token_string(tokens: &str) -> String {
    tokens
        .replace(" . ", ".")
        .replace(" :: ", "::")
        .replace("( ", "(")
        .replace(" )", ")")
        .replace(" ,", ",")
        .replace("! ", "!")
}

/// Whether a struct has `Structly` in one of its `#[derive(...)]` lists.
pub fn derives_structly(item: &syn::ItemStruct) -> bool {
    let mut found = false;
    for attr in &item.attrs {
        if !attr.path().is_ident("derive") {
            continue;
        }
        let _ = attr.parse_nested_meta(|meta| {
            if meta
                .path
                .segments
                .last()
                .is_some_and(|segment| segment.ident == "Structly")
            {
                found = true;
            }
            Ok(())
        });
    }
    found
}

/// Build a [`StructDoc`] from a parsed struct. `resolve_nested` is consulted
/// for each `#[structly(nested)]` field and may return the nested type's own
/// documentation (or `None` when it cannot be found).
pub fn build_struct_doc<F>(item: &syn::ItemStruct, resolve_nested: &mut F) -> syn::Result<StructDoc>
where
    F: FnMut(&syn::Type) -> Option<StructDoc>,
{
    let fields = match &item.fields {
        syn::Fields::Named(named) => &named.named,
        _ => {
            return Err(syn::Error::new_spanned(
                &item.ident,
                "only named fields are supported",
            ));
        }
    };

    let mut docs = Vec::new();
    for field in fields {
        let config = parse_field_config(field)?;
        let field_name = field.ident.as_ref().unwrap().to_string();
        let section = if config.nested {
            resolve_nested(&field.ty)
        } else {
            None
        };

        docs.push(FieldDoc {
            name: config.name.unwrap_or_else(|| field_name.clone()),
            field: field_name,
            description: config.description.unwrap_or_default(),
            mode: config.mode.unwrap_or_else(|| "all".to_string()),
            rules: config
                .rules
                .iter()
                .map(|rule| RuleDoc {
                    condition: condition_source(&rule.condition),
                    reason: rule.reason.clone(),
                })
                .collect(),
            nested: config.nested,
            section,
        });
    }

    Ok(StructDoc {
        name: item.ident.to_string(),
        fields: docs,
    })
}

// ---------------------------------------------------------------------------
// Markdown rendering
// ---------------------------------------------------------------------------

/// Render a [`StructDoc`] as markdown: title, table of contents, then
/// per-field sections with nested structs as sub-sections with dotted paths.
pub fn render_markdown(doc: &StructDoc) -> String {
    let mut out = String::new();
    out.push_str(&format!("# {}\n\n## Contents\n\n", doc.name));
    render_toc(&doc.fields, "", 0, &mut out);
    render_fields(&doc.fields, "", 2, &mut out);
    out
}

/// The heading text for a field at `path` (shared by the body and the TOC).
fn field_heading(field: &FieldDoc, path: &str) -> String {
    if let Some(section) = &field.section {
        format!("{path} (section: {})", section.name)
    } else if field.name != field.field {
        format!("{path} ({})", field.name)
    } else {
        path.to_string()
    }
}

/// GitHub-style anchor slug for a heading: lowercase, spaces to `-`, and
/// everything except letters, digits, `_`, and `-` removed.
fn heading_anchor(heading: &str) -> String {
    heading
        .to_lowercase()
        .chars()
        .filter_map(|c| match c {
            'a'..='z' | '0'..='9' | '_' | '-' => Some(c),
            ' ' => Some('-'),
            _ => None,
        })
        .collect()
}

fn render_toc(fields: &[FieldDoc], prefix: &str, indent: usize, out: &mut String) {
    for field in fields {
        let path = format!("{prefix}{}", field.field);
        let heading = field_heading(field, &path);
        out.push_str(&format!(
            "{}- [{heading}](#{})\n",
            "  ".repeat(indent),
            heading_anchor(&heading)
        ));
        if let Some(section) = &field.section {
            render_toc(&section.fields, &format!("{path}."), indent + 1, out);
        }
    }
}

fn render_fields(fields: &[FieldDoc], prefix: &str, level: usize, out: &mut String) {
    let hashes = "#".repeat(level.min(6));

    for field in fields {
        let path = format!("{prefix}{}", field.field);

        out.push('\n');
        out.push_str(&format!("{hashes} {}\n", field_heading(field, &path)));

        if !field.description.is_empty() {
            out.push('\n');
            out.push_str(&field.description);
            out.push('\n');
        }

        if field.description.is_empty() && field.rules.is_empty() && !field.nested {
            out.push_str("\n_No details added._\n");
        }

        if !field.rules.is_empty() {
            out.push_str("\n**Validation:**\n\n");
            if field.rules.len() >= 2 {
                match field.mode.as_str() {
                    "fail_fast" => out.push_str(
                        "_Rules are checked in order. Only the first failure is reported (`fail_fast`)._\n\n",
                    ),
                    "any" => out.push_str(
                        "_At least one of the following must be satisfied (`any`)._\n\n",
                    ),
                    _ => {}
                }
            }
            for rule in &field.rules {
                out.push_str(&format!(
                    "- Required when `{}`: {}\n",
                    rule.condition, rule.reason
                ));
            }
        }

        if field.nested {
            if let Some(section) = &field.section {
                render_fields(&section.fields, &format!("{path}."), level + 1, out);
            } else {
                out.push_str("\n_Nested section - type not found in the parsed sources._\n");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_struct(src: &str) -> syn::ItemStruct {
        syn::parse_str(src).expect("fixture should parse")
    }

    #[test]
    fn builds_field_docs_with_defaults_and_overrides() {
        let item = parse_struct(
            r#"
            #[derive(Structly)]
            struct Demo {
                #[structly(name = "Label", description = "A label.")]
                #[structly_if(when = self.enabled, reason = "label required when enabled")]
                label: Option<String>,

                enabled: bool,
            }
            "#,
        );

        let doc = build_struct_doc(&item, &mut |_| None).unwrap();
        assert_eq!(doc.name, "Demo");
        assert_eq!(doc.fields.len(), 2);

        let label = &doc.fields[0];
        assert_eq!(label.field, "label");
        assert_eq!(label.name, "Label");
        assert_eq!(label.description, "A label.");
        assert_eq!(label.mode, "all");
        assert_eq!(label.rules.len(), 1);
        assert_eq!(label.rules[0].condition, "self.enabled");
        assert_eq!(label.rules[0].reason, "label required when enabled");

        let enabled = &doc.fields[1];
        assert_eq!(enabled.field, "enabled");
        assert_eq!(enabled.name, "enabled");
        assert!(enabled.rules.is_empty());
    }

    #[test]
    fn falls_back_to_doc_comments_for_descriptions() {
        let item = parse_struct(
            r#"
            #[derive(Structly)]
            struct Demo {
                /// Multiline docs are kept as-is,
                /// including this second line.
                ///
                /// And this paragraph.
                documented: bool,

                /// Doc comment loses.
                #[structly(description = "Explicit wins.")]
                explicit: bool,

                bare: bool,
            }
            "#,
        );

        let doc = build_struct_doc(&item, &mut |_| None).unwrap();
        assert_eq!(
            doc.fields[0].description,
            "Multiline docs are kept as-is,\nincluding this second line.\n\nAnd this paragraph."
        );
        assert_eq!(doc.fields[1].description, "Explicit wins.");
        assert_eq!(doc.fields[2].description, "");
    }

    #[test]
    fn resolves_nested_sections_through_the_callback() {
        let inner = parse_struct(
            r#"
            #[derive(Structly)]
            struct Database {
                #[structly_if(when = self.ssl, reason = "cert required when ssl is on")]
                cert: Option<String>,
                ssl: bool,
            }
            "#,
        );
        let outer = parse_struct(
            r#"
            #[derive(Structly)]
            struct AppConfig {
                #[structly(nested)]
                database: Database,
            }
            "#,
        );

        let doc = build_struct_doc(&outer, &mut |ty| {
            let syn::Type::Path(path) = ty else { return None };
            (path.path.segments.last()?.ident == "Database")
                .then(|| build_struct_doc(&inner, &mut |_| None).unwrap())
        })
            .unwrap();

        let database = &doc.fields[0];
        assert!(database.nested);
        let section = database.section.as_ref().expect("nested doc resolved");
        assert_eq!(section.name, "Database");
        assert_eq!(section.fields[0].rules[0].condition, "self.ssl");
    }

    #[test]
    fn renders_markdown_with_dotted_nested_paths() {
        let item = parse_struct(
            r#"
            #[derive(Structly)]
            struct Database {
                #[structly(description = "PEM certificate.")]
                #[structly_if(when = self.ssl, reason = "cert required when ssl is on")]
                cert: Option<String>,
                ssl: bool,
            }
            "#,
        );
        let nested = build_struct_doc(&item, &mut |_| None).unwrap();

        let doc = StructDoc {
            name: "AppConfig".to_string(),
            fields: vec![FieldDoc {
                field: "database".to_string(),
                name: "database".to_string(),
                description: "Database settings.".to_string(),
                mode: "all".to_string(),
                rules: vec![],
                nested: true,
                section: Some(nested),
            }],
        };

        let md = render_markdown(&doc);
        assert!(md.contains("# AppConfig"));
        assert!(md.contains("## database (section: Database)"));
        assert!(md.contains("Database settings."));
        assert!(md.contains("### database.cert"));
        assert!(md.contains("PEM certificate."));
        assert!(md.contains("- Required when `self.ssl`: cert required when ssl is on"));
    }

    #[test]
    fn notes_the_mode_for_multi_rule_fields() {
        let item = parse_struct(
            r#"
            #[derive(Structly)]
            struct Payment {
                #[structly(mode = "any")]
                #[structly_if(when = self.card, reason = "card number required")]
                #[structly_if(when = self.bank, reason = "account number required")]
                value: Option<String>,
                card: bool,
                bank: bool,
            }
            "#,
        );

        let md = render_markdown(&build_struct_doc(&item, &mut |_| None).unwrap());
        assert!(md.contains("_At least one of the following must be satisfied (`any`)._"));
        assert!(md.contains("- Required when `self.card`: card number required"));
        assert!(md.contains("- Required when `self.bank`: account number required"));
    }

    #[test]
    fn renders_a_linked_table_of_contents() {
        let inner = parse_struct(
            r#"
            #[derive(Structly)]
            struct Database {
                #[structly(name = "Certificate")]
                cert_path: Option<String>,
            }
            "#,
        );
        let nested = build_struct_doc(&inner, &mut |_| None).unwrap();

        let doc = StructDoc {
            name: "AppConfig".to_string(),
            fields: vec![
                FieldDoc {
                    field: "database".to_string(),
                    name: "database".to_string(),
                    description: String::new(),
                    mode: "all".to_string(),
                    rules: vec![],
                    nested: true,
                    section: Some(nested),
                },
                FieldDoc {
                    field: "debug".to_string(),
                    name: "debug".to_string(),
                    description: String::new(),
                    mode: "all".to_string(),
                    rules: vec![],
                    nested: false,
                    section: None,
                },
            ],
        };

        let md = render_markdown(&doc);
        assert!(md.contains("## Contents"));
        // Nested entries are indented under their section. Anchors match the
        // GitHub slugs of the generated headings (dots removed, spaces to `-`).
        assert!(md.contains("- [database (section: Database)](#database-section-database)"));
        assert!(md.contains("  - [database.cert_path (Certificate)](#databasecert_path-certificate)"));
        assert!(md.contains("- [debug](#debug)"));
        // The linked headings themselves exist.
        assert!(md.contains("## database (section: Database)"));
        assert!(md.contains("### database.cert_path (Certificate)"));
    }

    #[test]
    fn notes_fields_without_description_or_rules() {
        let item = parse_struct(
            r#"
            #[derive(Structly)]
            struct Demo {
                #[structly(description = "Documented.")]
                documented: bool,
                bare: bool,
            }
            "#,
        );

        let md = render_markdown(&build_struct_doc(&item, &mut |_| None).unwrap());
        assert!(md.contains("## bare\n\n_No details added._"));
        // Documented fields don't get the placeholder.
        assert!(!md.contains("Documented.\n\n_No details added._"));
    }

    #[test]
    fn detects_the_structly_derive() {
        let with = parse_struct("#[derive(Debug, Structly)] struct A { x: bool }");
        let without = parse_struct("#[derive(Debug, Clone)] struct B { x: bool }");
        assert!(derives_structly(&with));
        assert!(!derives_structly(&without));
    }

    #[test]
    fn marks_unresolvable_nested_sections() {
        let item = parse_struct(
            r#"
            #[derive(Structly)]
            struct AppConfig {
                #[structly(nested)]
                database: Database,
            }
            "#,
        );

        let md = render_markdown(&build_struct_doc(&item, &mut |_| None).unwrap());
        assert!(md.contains("_Nested section - type not found in the parsed sources._"));
    }
}

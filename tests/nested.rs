//! `#[structly(nested)]` recurses into a sub-struct's `verify()`, prefixing each
//! error's field with the section name (`section.field`). List fields (`Vec<T>`,
//! arrays, slices) verify element-wise with indexed paths (`section[0].field`).

use structly::{Structly, Verify};

#[allow(unused)]
#[derive(Structly)]
struct Database {
    #[structly_if(when = self.ssl, reason = "cert required when ssl is on")]
    cert: Option<String>,
    ssl: bool,
}

#[allow(unused)]
#[derive(Structly)]
struct AppConfig {
    #[structly(nested)]
    database: Database,

    #[structly_if(when = self.public, reason = "domain required for public apps")]
    domain: Option<String>,
    public: bool,
}

#[test]
fn nested_errors_are_reported_with_a_dotted_path() {
    let cfg = AppConfig {
        database: Database { cert: None, ssl: true },
        domain: Some("example.com".into()),
        public: true,
    };
    let err = cfg.verify().expect_err("expected a nested validation error");
    assert_eq!(err.len(), 1);
    assert_eq!(err[0].field, "database.cert");
    assert_eq!(err[0].reason, "cert required when ssl is on");
}

#[test]
fn nested_passes_when_the_subsection_is_valid() {
    let cfg = AppConfig {
        database: Database { cert: Some("cert.pem".into()), ssl: true },
        domain: Some("example.com".into()),
        public: true,
    };
    assert!(cfg.verify().is_ok());
}

#[test]
fn own_and_nested_errors_surface_together_in_declaration_order() {
    // `database` (declared first) is invalid, and so is `domain`.
    let cfg = AppConfig {
        database: Database { cert: None, ssl: true },
        domain: None,
        public: true,
    };
    let err = cfg.verify().expect_err("expected validation errors");
    assert_eq!(err.len(), 2);
    assert_eq!(err[0].field, "database.cert");
    assert_eq!(err[1].field, "domain");
}

#[allow(unused)]
#[derive(Structly)]
struct Outer {
    #[structly(nested)]
    middle: Middle,
}

#[allow(unused)]
#[derive(Structly)]
struct Middle {
    #[structly(nested)]
    inner: Inner,
}

#[allow(unused)]
#[derive(Structly)]
struct Inner {
    #[structly_if(when = self.required, reason = "value is required")]
    value: Option<String>,
    required: bool,
}

#[allow(unused)]
#[derive(Structly)]
struct CommitCategory {
    #[structly_if(when = self.custom, reason = "name required for custom categories")]
    name: Option<String>,
    custom: bool,
}

#[allow(unused)]
#[derive(Structly)]
struct Changelog {
    #[structly(nested)]
    categories: Vec<CommitCategory>,
}

#[test]
fn nested_lists_report_indexed_paths() {
    let changelog = Changelog {
        categories: vec![
            CommitCategory { name: Some("Features".into()), custom: true },
            CommitCategory { name: None, custom: true },
            CommitCategory { name: None, custom: false },
            CommitCategory { name: None, custom: true },
        ],
    };
    let err = changelog.verify().expect_err("expected indexed validation errors");
    assert_eq!(err.len(), 2);
    assert_eq!(err[0].field, "categories[1].name");
    assert_eq!(err[1].field, "categories[3].name");
    assert_eq!(err[0].reason, "name required for custom categories");
}

#[test]
fn nested_lists_pass_when_all_elements_are_valid() {
    let changelog = Changelog {
        categories: vec![
            CommitCategory { name: Some("Features".into()), custom: true },
            CommitCategory { name: None, custom: false },
        ],
    };
    assert!(changelog.verify().is_ok());

    let empty = Changelog { categories: vec![] };
    assert!(empty.verify().is_ok());
}

#[allow(unused)]
#[derive(Structly)]
struct FixedSlots {
    #[structly(nested)]
    slots: [Inner; 2],
}

#[test]
fn nested_arrays_report_indexed_paths() {
    let fixed = FixedSlots {
        slots: [
            Inner { value: None, required: false },
            Inner { value: None, required: true },
        ],
    };
    let err = fixed.verify().expect_err("expected an indexed validation error");
    assert_eq!(err.len(), 1);
    assert_eq!(err[0].field, "slots[1].value");
}

#[allow(unused)]
#[derive(Structly)]
struct Grouped {
    #[structly(nested)]
    groups: Vec<Middle>,
}

#[test]
fn lists_of_nested_sections_compose() {
    let grouped = Grouped {
        groups: vec![Middle { inner: Inner { value: None, required: true } }],
    };
    let err = grouped.verify().expect_err("expected a deeply nested indexed error");
    assert_eq!(err.len(), 1);
    assert_eq!(err[0].field, "groups[0].inner.value");
}

#[test]
fn nesting_composes_to_multiple_levels() {
    let cfg = Outer {
        middle: Middle { inner: Inner { value: None, required: true } },
    };
    let err = cfg.verify().expect_err("expected a deeply nested error");
    assert_eq!(err.len(), 1);
    assert_eq!(err[0].field, "middle.inner.value");
}

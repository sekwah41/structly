//! `#[structly(nested)]` recurses into a sub-struct's `verify()`, prefixing each
//! error's field with the section name (`section.field`).

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

#[test]
fn nesting_composes_to_multiple_levels() {
    let cfg = Outer {
        middle: Middle { inner: Inner { value: None, required: true } },
    };
    let err = cfg.verify().expect_err("expected a deeply nested error");
    assert_eq!(err.len(), 1);
    assert_eq!(err[0].field, "middle.inner.value");
}

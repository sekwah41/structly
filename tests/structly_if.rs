//! `#[structly_if(...)]` conditional rules that gate `verify()` on other fields.

use structly::{Structly, Verify};

#[allow(unused)]
#[derive(Structly)]
struct Config {
    #[structly_if(when = self.enabled, reason = "name required when enabled")]
    name: Option<String>,

    enabled: bool,
}

#[test]
fn passes_when_condition_false() {
    let config = Config { name: None, enabled: false };
    assert!(config.verify().is_ok());
}

#[test]
fn fails_when_condition_true_and_field_missing() {
    let config = Config { name: None, enabled: true };
    let err = config.verify().expect_err("expected a validation error");
    assert_eq!(err.len(), 1);
    assert_eq!(err[0].field, "name");
    assert_eq!(err[0].reason, "name required when enabled");
}

#[test]
fn passes_when_condition_true_and_field_present() {
    let config = Config { name: Some("db".into()), enabled: true };
    assert!(config.verify().is_ok());
}

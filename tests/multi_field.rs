//! Errors from rules across several fields are collected in field declaration order.

use structly::{Structly, Verify};

#[allow(unused)]
#[derive(Structly)]
struct Server {
    #[structly_if(when = self.tls, reason = "cert required when tls is on")]
    cert: Option<String>,

    #[structly_if(when = self.tls, reason = "key required when tls is on")]
    key: Option<String>,

    tls: bool,
}

#[test]
fn errors_from_multiple_fields_are_collected_in_declaration_order() {
    let server = Server { cert: None, key: None, tls: true };
    let err = server.verify().expect_err("expected validation errors");
    assert_eq!(err.len(), 2);
    assert_eq!(err[0].field, "cert");
    assert_eq!(err[0].reason, "cert required when tls is on");
    assert_eq!(err[1].field, "key");
    assert_eq!(err[1].reason, "key required when tls is on");
}

#[test]
fn only_failing_fields_are_reported() {
    let server = Server { cert: Some("cert.pem".into()), key: None, tls: true };
    let err = server.verify().expect_err("expected a validation error");
    assert_eq!(err.len(), 1);
    assert_eq!(err[0].field, "key");
}

#[test]
fn passes_when_all_conditional_fields_are_present() {
    let server = Server {
        cert: Some("cert.pem".into()),
        key: Some("key.pem".into()),
        tls: true,
    };
    assert!(server.verify().is_ok());
}

#[allow(unused)]
#[derive(Structly)]
struct Retry {
    #[structly_if(when = self.max_retries > 3, reason = "backoff required for aggressive retries")]
    backoff_ms: Option<u64>,

    max_retries: u32,
}

#[test]
fn conditions_can_be_arbitrary_expressions_over_self() {
    let retry = Retry { backoff_ms: None, max_retries: 5 };
    let err = retry.verify().expect_err("expected a validation error");
    assert_eq!(err[0].field, "backoff_ms");
    assert_eq!(err[0].reason, "backoff required for aggressive retries");

    let retry = Retry { backoff_ms: None, max_retries: 2 };
    assert!(retry.verify().is_ok());
}

//! How multiple `structly_if` rules on one field combine per `mode`

use structly::{Structly, Verify};

#[allow(unused)]
#[derive(Structly)]
struct AllMode {
    #[structly(mode = "all")]
    #[structly_if(when = self.https, reason = "cert required for https")]
    #[structly_if(when = self.mutual_tls, reason = "cert required for mutual tls")]
    cert_path: Option<String>,

    https: bool,
    mutual_tls: bool,
}

#[test]
fn all_mode_collects_every_failing_rule() {
    let config = AllMode { cert_path: None, https: true, mutual_tls: true };
    let err = config.verify().expect_err("expected validation errors");
    assert_eq!(err.len(), 2);
    assert_eq!(err[0].reason, "cert required for https");
    assert_eq!(err[1].reason, "cert required for mutual tls");
}

#[test]
fn all_mode_reports_only_the_rules_that_fail() {
    let config = AllMode { cert_path: None, https: true, mutual_tls: false };
    let err = config.verify().expect_err("expected a validation error");
    assert_eq!(err.len(), 1);
    assert_eq!(err[0].reason, "cert required for https");
}

#[allow(unused)]
#[derive(Structly)]
struct DefaultMode {
    #[structly_if(when = self.https, reason = "cert required for https")]
    #[structly_if(when = self.mutual_tls, reason = "cert required for mutual tls")]
    cert_path: Option<String>,

    https: bool,
    mutual_tls: bool,
}

#[test]
fn default_mode_behaves_like_all() {
    let config = DefaultMode { cert_path: None, https: true, mutual_tls: true };
    let err = config.verify().expect_err("expected validation errors");
    assert_eq!(err.len(), 2);
}

#[allow(unused)]
#[derive(Structly)]
struct FailFastMode {
    #[structly(mode = "fail_fast")]
    #[structly_if(when = self.https, reason = "cert required for https")]
    #[structly_if(when = self.mutual_tls, reason = "cert required for mutual tls")]
    cert_path: Option<String>,

    https: bool,
    mutual_tls: bool,
}

#[test]
fn fail_fast_mode_stops_at_the_first_failing_rule() {
    let config = FailFastMode { cert_path: None, https: true, mutual_tls: true };
    let err = config.verify().expect_err("expected a validation error");
    assert_eq!(err.len(), 1);
    assert_eq!(err[0].reason, "cert required for https");
}

#[allow(unused)]
#[derive(Structly)]
struct AnyMode {
    #[structly(mode = "any")]
    #[structly_if(when = self.card_payment, reason = "value required for card payments")]
    #[structly_if(when = self.bank_payment, reason = "value required for bank payments")]
    value: Option<String>,

    card_payment: bool,
    bank_payment: bool,
}

#[test]
fn any_mode_passes_when_at_least_one_rule_passes() {
    let config = AnyMode { value: None, card_payment: true, bank_payment: false };
    assert!(config.verify().is_ok());
}

#[test]
fn any_mode_fails_when_every_rule_fails() {
    let config = AnyMode { value: None, card_payment: true, bank_payment: true };
    let err = config.verify().expect_err("expected validation errors");
    assert!(!err.is_empty());
    assert_eq!(err[0].field, "value");
}

#[test]
fn any_mode_passes_when_the_field_is_present() {
    let config = AnyMode {
        value: Some("visa".into()),
        card_payment: true,
        bank_payment: true,
    };
    assert!(config.verify().is_ok());
}

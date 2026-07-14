//! Three `structly_if` rules per field in each mode, with passing/failing rules
//! interleaved (e.g. `fail_fast` reports the first rule that *fails*).

use structly::{Structly, Verify};

#[allow(unused)]
#[derive(Structly)]
struct AllThree {
    #[structly(mode = "all")]
    #[structly_if(when = self.a, reason = "a")]
    #[structly_if(when = self.b, reason = "b")]
    #[structly_if(when = self.c, reason = "c")]
    field: Option<String>,

    a: bool,
    b: bool,
    c: bool,
}

#[test]
fn all_mode_reports_the_failing_rules_in_order_skipping_the_gaps() {
    // Middle rule's condition is false, so only `a` and `c` fail.
    let cfg = AllThree { field: None, a: true, b: false, c: true };
    let err = cfg.verify().expect_err("expected validation errors");
    assert_eq!(err.len(), 2);
    assert_eq!(err[0].reason, "a");
    assert_eq!(err[1].reason, "c");
}

#[test]
fn all_mode_passes_when_no_conditions_hold() {
    let cfg = AllThree { field: None, a: false, b: false, c: false };
    assert!(cfg.verify().is_ok());
}

#[test]
fn all_mode_passes_when_the_field_is_present() {
    let cfg = AllThree { field: Some("x".into()), a: true, b: true, c: true };
    assert!(cfg.verify().is_ok());
}

#[allow(unused)]
#[derive(Structly)]
struct FailFastThree {
    #[structly(mode = "fail_fast")]
    #[structly_if(when = self.a, reason = "a")]
    #[structly_if(when = self.b, reason = "b")]
    #[structly_if(when = self.c, reason = "c")]
    field: Option<String>,

    a: bool,
    b: bool,
    c: bool,
}

#[test]
fn fail_fast_reports_the_first_failing_rule_not_the_first_rule() {
    // `a`'s condition is false, so the first *failure* is `b`.
    let cfg = FailFastThree { field: None, a: false, b: true, c: true };
    let err = cfg.verify().expect_err("expected a validation error");
    assert_eq!(err.len(), 1);
    assert_eq!(err[0].reason, "b");
}

#[test]
fn fail_fast_stops_at_the_very_first_rule_when_it_fails() {
    let cfg = FailFastThree { field: None, a: true, b: true, c: true };
    let err = cfg.verify().expect_err("expected a validation error");
    assert_eq!(err.len(), 1);
    assert_eq!(err[0].reason, "a");
}

#[test]
fn fail_fast_passes_when_no_conditions_hold() {
    let cfg = FailFastThree { field: None, a: false, b: false, c: false };
    assert!(cfg.verify().is_ok());
}

#[allow(unused)]
#[derive(Structly)]
struct AnyThree {
    #[structly(mode = "any")]
    #[structly_if(when = self.a, reason = "a")]
    #[structly_if(when = self.b, reason = "b")]
    #[structly_if(when = self.c, reason = "c")]
    field: Option<String>,

    a: bool,
    b: bool,
    c: bool,
}

#[test]
fn any_mode_passes_when_a_single_rule_passes() {
    // `a` and `b` fail, but `c`'s condition is false so that rule passes.
    let cfg = AnyThree { field: None, a: true, b: true, c: false };
    assert!(cfg.verify().is_ok());
}

#[test]
fn any_mode_fails_only_when_every_rule_fails() {
    let cfg = AnyThree { field: None, a: true, b: true, c: true };
    let err = cfg.verify().expect_err("expected validation errors");
    // Every rule failed, so they collapse into one combined, human-readable error.
    assert_eq!(err.len(), 1);
    assert_eq!(err[0].field, "field");
    assert_eq!(err[0].reason, "One of the following must be true:\n - a\n - b\n - c");
}

#[test]
fn any_mode_passes_when_the_field_is_present() {
    let cfg = AnyThree { field: Some("x".into()), a: true, b: true, c: true };
    assert!(cfg.verify().is_ok());
}

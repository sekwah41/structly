//! Core derive behaviour: the `Structly` derive wires up the `Verify` trait.

use structly::{Structly, Verify};

#[allow(unused)]
#[derive(Structly)]
struct Demo {
    #[structly(name = "Label", description = "This is a fun label description")]
    label: Option<String>,
}

#[test]
fn verify_returns_ok_when_no_rules_defined() {
    let demo = Demo { label: None };
    assert!(demo.verify().is_ok());
}

#[test]
fn derive_implements_verify_trait() {
    fn assert_implements_verify<T: Verify>(_: &T) {}

    let demo = Demo { label: Some("hello".into()) };
    assert_implements_verify(&demo);
}

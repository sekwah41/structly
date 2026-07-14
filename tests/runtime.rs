//! Behaviour of the runtime support types in the `structly` crate itself.

use structly::{FieldMeta, ValidationError};

#[test]
fn validation_error_displays_field_and_reason() {
    let err = ValidationError::new("name", "name required when enabled");
    assert_eq!(err.to_string(), "name: name required when enabled");
}

#[test]
fn validation_error_is_clone_and_debug() {
    fn assert_clone_debug<T: Clone + std::fmt::Debug>(_: T) {}
    assert_clone_debug(ValidationError::new("a", "b"));
}

#[test]
fn field_meta_is_copy_and_debug() {
    fn assert_copy_debug<T: Copy + std::fmt::Debug>(_: T) {}
    assert_copy_debug(FieldMeta { field: "label", name: "Label", description: "" });
}

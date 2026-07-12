//! Compile-fail tests: each `tests/compile_fail/*.rs` must fail to compile,
//! matched against its `.stderr` snapshot. Regenerate snapshots with
//! `TRYBUILD=overwrite cargo test --test compile_fail`.

#[test]
fn compile_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/*.rs");
}

//! The same two `structly_if` rules on one field, under each `mode`, so you can
//! see how the surfaced errors differ.
//!
//! The `value` field is required when paying by card *and/or* by bank. Each
//! struct below carries the identical pair of rules — only the `mode` changes.
//!
//! Run with `cargo run --example multiple_ifs`.

use structly::{Structly, ValidationError, Verify};

#[allow(unused)]
#[derive(Structly)]
struct AllMode {
    // `all` is the default; every failing rule surfaces, in order.
    #[structly_if(when = self.card, reason = "card number required for card payments")]
    #[structly_if(when = self.bank, reason = "account number required for bank payments")]
    value: Option<String>,
    card: bool,
    bank: bool,
}

#[allow(unused)]
#[derive(Structly)]
struct FailFastMode {
    #[structly(mode = "fail_fast")]
    #[structly_if(when = self.card, reason = "card number required for card payments")]
    #[structly_if(when = self.bank, reason = "account number required for bank payments")]
    value: Option<String>,
    card: bool,
    bank: bool,
}

#[allow(unused)]
#[derive(Structly)]
struct AnyMode {
    // Valid as long as *one* rule passes; only errors when every rule fails.
    #[structly(mode = "any")]
    #[structly_if(when = self.card, reason = "card number required for card payments")]
    #[structly_if(when = self.bank, reason = "account number required for bank payments")]
    value: Option<String>,
    card: bool,
    bank: bool,
}

fn show(mode: &str, result: Result<(), Vec<ValidationError>>) {
    match result {
        Ok(()) => println!("  {mode:9} -> ok"),
        Err(errors) => {
            println!("  {mode:9} -> {} error(s):", errors.len());
            for err in errors {
                // `ValidationError` prints as "field: reason"; the `any` message
                // spans several lines, so indent each one.
                for line in err.to_string().lines() {
                    println!("      {line}");
                }
            }
        }
    }
}

fn main() {
    // Both conditions hold and `value` is missing, so both rules fail.
    println!("card = true, bank = true, value = None");
    show("all", AllMode { value: None, card: true, bank: true }.verify());
    show("fail_fast", FailFastMode { value: None, card: true, bank: true }.verify());
    show("any", AnyMode { value: None, card: true, bank: true }.verify());

    // Only the card rule's condition holds, so the bank rule passes vacuously.
    println!("\ncard = true, bank = false, value = None");
    show("all", AllMode { value: None, card: true, bank: false }.verify());
    show("fail_fast", FailFastMode { value: None, card: true, bank: false }.verify());
    show("any", AnyMode { value: None, card: true, bank: false }.verify());

    // The field is present, so nothing is required regardless of mode.
    println!("\ncard = true, bank = true, value = Some");
    show("all", AllMode { value: Some("x".into()), card: true, bank: true }.verify());
    show("fail_fast", FailFastMode { value: Some("x".into()), card: true, bank: true }.verify());
    show("any", AnyMode { value: Some("x".into()), card: true, bank: true }.verify());
}

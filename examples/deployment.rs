//! Two rules on one field, an expression condition, and `mode = "fail_fast"`.
//!
//! Example Scenario
//! A load balancer is required if there is more than one replica *or* the
//! deployment is public. `fail_fast` reports only the first rule that fails, so
//! you fix one problem at a time.
//!
//! Run with `cargo run --example deployment`.

use structly::{Structly, Verify};

#[allow(unused)]
#[derive(Structly)]
struct Deployment {
    #[structly(mode = "fail_fast")]
    #[structly_if(when = self.replicas > 1, reason = "a load balancer is required with more than one replica")]
    #[structly_if(when = self.public, reason = "a load balancer is required for public deployments")]
    load_balancer: Option<String>,

    replicas: u32,
    public: bool,
}

fn main() {
    let cases = [
        // Both rules would fail, but fail_fast reports only the first.
        ("3 replicas, public, no LB", Deployment { load_balancer: None, replicas: 3, public: true }),
        // First rule passes (1 replica), so the public rule is the first failure.
        ("1 replica, public, no LB", Deployment { load_balancer: None, replicas: 1, public: true }),
        ("3 replicas, private, LB set", Deployment { load_balancer: Some("lb-01".into()), replicas: 3, public: false }),
        ("1 replica, private", Deployment { load_balancer: None, replicas: 1, public: false }),
    ];

    for (label, deployment) in cases {
        match deployment.verify() {
            Ok(()) => println!("[{label}] valid"),
            Err(errors) => {
                println!("[{label}] invalid:");
                for err in errors {
                    println!("  - {err}");
                }
            }
        }
    }
}

//! Nested sections with `#[structly(nested)]`, as you'd get from a layered
//! JSON/TOML config. Errors from a subsection surface with a dotted path.
//!
//! Run with `cargo run --example nested_config`.

use structly::{Structly, Verify};

#[allow(unused)]
#[derive(Structly)]
struct Database {
    #[structly_if(when = self.ssl, reason = "a certificate is required when SSL is enabled")]
    cert_path: Option<String>,
    ssl: bool,
}

#[allow(unused)]
#[derive(Structly)]
struct Server {
    #[structly_if(when = self.public, reason = "a domain is required for public servers")]
    domain: Option<String>,
    public: bool,
}

#[allow(unused)]
#[derive(Structly)]
struct AppConfig {
    // Each section derives `Structly` and is recursed into with `nested`.
    #[structly(nested)]
    database: Database,

    #[structly(nested)]
    server: Server,
}

fn main() {
    let config = AppConfig {
        database: Database { cert_path: None, ssl: true },
        server: Server { domain: None, public: true },
    };

    match config.verify() {
        Ok(()) => println!("config is valid"),
        Err(errors) => {
            println!("config is invalid:");
            for err in errors {
                // Prints e.g. "database.cert_path: a certificate is required ..."
                println!("  - {err}");
            }
        }
    }
}

//! Validating a server config: enabling TLS requires both a cert and a key.
//!
//! Run with `cargo run --example server_config`.

use structly::{Structly, Verify};

#[allow(unused)]
#[derive(Structly)]
struct ServerConfig {
    #[structly(name = "Host", description = "Address the server binds to")]
    host: Option<String>,

    #[structly(description = "PEM certificate, required when TLS is on")]
    #[structly_if(when = self.tls, reason = "a certificate is required when TLS is enabled")]
    cert_path: Option<String>,

    #[structly(description = "Private key, required when TLS is on")]
    #[structly_if(when = self.tls, reason = "a private key is required when TLS is enabled")]
    key_path: Option<String>,

    tls: bool,
}

fn main() {
    let cases = [
        (
            "TLS on, files missing",
            ServerConfig { host: Some("0.0.0.0".into()), cert_path: None, key_path: None, tls: true },
        ),
        (
            "TLS on, fully configured",
            ServerConfig {
                host: Some("0.0.0.0".into()),
                cert_path: Some("/etc/ssl/cert.pem".into()),
                key_path: Some("/etc/ssl/key.pem".into()),
                tls: true,
            },
        ),
        (
            "TLS off",
            ServerConfig { host: Some("0.0.0.0".into()), cert_path: None, key_path: None, tls: false },
        ),
    ];

    for (label, config) in cases {
        match config.verify() {
            Ok(()) => println!("[{label}] valid"),
            Err(errors) => {
                println!("[{label}] invalid:");
                for err in errors {
                    // `ValidationError` prints as "field: reason".
                    println!("  - {err}");
                }
            }
        }
    }
}

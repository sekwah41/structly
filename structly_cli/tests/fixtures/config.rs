//! Fixture for the CLI tests - parsed, never compiled.

use structly::{Structly, Verify};

#[derive(Structly)]
struct Database {
    #[structly(name = "Certificate", description = "PEM certificate used for SSL.")]
    #[structly_if(when = self.ssl, reason = "cert required when ssl is on")]
    cert_path: Option<String>,

    ssl: bool,
}

#[derive(Structly)]
struct Server {
    #[structly_if(when = self.public, reason = "domain required for public servers")]
    domain: Option<String>,

    public: bool,
}

#[derive(Structly)]
struct CommitCategory {
    #[structly_if(when = self.custom, reason = "name required for custom categories")]
    name: Option<String>,

    custom: bool,
}

#[derive(Structly)]
struct AppConfig {
    #[structly(description = "Database connection settings.", nested)]
    database: Database,

    #[structly(nested)]
    server: Server,

    #[structly(nested)]
    categories: Vec<CommitCategory>,
}

#[derive(Debug)]
struct NotStructly {
    ignored: bool,
}

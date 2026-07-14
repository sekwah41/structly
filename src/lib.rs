pub use structly_macros::Structly;

#[derive(Debug, Clone, Copy)]
pub struct FieldMeta {
    pub field: &'static str,
    pub name: &'static str,
    pub description: &'static str,
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Field path, dotted for nested structs (e.g. `database.cert`).
    pub field: String,
    pub reason: &'static str,
}

impl ValidationError {
    /// Construct an error; `field` accepts a bare `&str` so callers avoid `.into()`.
    pub fn new(field: impl Into<String>, reason: &'static str) -> Self {
        Self { field: field.into(), reason }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.reason)
    }
}

pub trait Verify {
    fn verify(&self) -> Result<(), Vec<ValidationError>>;
}

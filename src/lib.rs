pub use structly_macros::Structly;

#[derive(Debug, Clone, Copy)]
pub struct FieldMeta {
    pub field: &'static str,
    pub name: &'static str,
    pub description: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct ValidationError {
    pub field: &'static str,
    pub reason: &'static str,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.reason)
    }
}

pub trait Verify {
    fn verify(&self) -> Result<(), Vec<ValidationError>>;
}

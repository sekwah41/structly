pub use structly_macros::Structly;

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

    /// Prepend a parent path segment: `cert` becomes `database.cert`, while an
    /// indexed path like `[0].cert` becomes `database[0].cert` (no dot).
    pub fn prefixed(mut self, parent: impl Into<String>) -> Self {
        let mut path = parent.into();
        if !self.field.starts_with('[') {
            path.push('.');
        }
        path.push_str(&self.field);
        self.field = path;
        self
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

/// Verify every element, prefixing its errors with `[index]` so a nested list
/// field reports paths like `categories[0].name`.
fn verify_items<'a, T: Verify + 'a>(
    items: impl Iterator<Item = &'a T>,
) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();
    for (index, item) in items.enumerate() {
        if let Err(item_errors) = item.verify() {
            let prefix = format!("[{index}]");
            errors.extend(item_errors.into_iter().map(|error| error.prefixed(&*prefix)));
        }
    }
    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

impl<T: Verify> Verify for Vec<T> {
    fn verify(&self) -> Result<(), Vec<ValidationError>> {
        verify_items(self.iter())
    }
}

impl<T: Verify> Verify for [T] {
    fn verify(&self) -> Result<(), Vec<ValidationError>> {
        verify_items(self.iter())
    }
}

impl<T: Verify, const N: usize> Verify for [T; N] {
    fn verify(&self) -> Result<(), Vec<ValidationError>> {
        verify_items(self.iter())
    }
}

//! Text with explicit accessibility intent and validation.

use std::fmt;

/// A trimmed, non-empty accessible name.
///
/// ```
/// use fc_dioxus::prelude::AccessibleName;
/// assert!(AccessibleName::try_from("Close").is_ok());
/// assert!(AccessibleName::try_from("  ").is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AccessibleName(String);

impl AccessibleName {
    /// The validated name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl TryFrom<&str> for AccessibleName {
    type Error = EmptyName;
    /// # Errors
    /// Returns `EmptyName` for empty or whitespace-only text.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.trim();
        if value.is_empty() {
            Err(EmptyName)
        } else {
            Ok(Self(value.to_owned()))
        }
    }
}
impl TryFrom<String> for AccessibleName {
    type Error = EmptyName;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}
impl fmt::Display for AccessibleName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// An accessible name contained no text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmptyName;
impl fmt::Display for EmptyName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("accessible names must contain text")
    }
}
impl std::error::Error for EmptyName {}

/// An image must explicitly be meaningful or decorative; there is no default.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Alt {
    /// A meaningful alternative.
    Text(AccessibleName),
    /// Intentionally empty alternative text.
    Decorative,
}

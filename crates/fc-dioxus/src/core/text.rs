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
    /// Backs [`name!`](crate::name), which has already rejected a blank literal at
    /// compile time. Not for direct use.
    #[doc(hidden)]
    #[must_use]
    pub fn from_checked_literal(text: &'static str) -> Self {
        let trimmed = text.trim();
        // The compile-time check sees ASCII whitespace only; a literal of other Unicode
        // spaces keeps its text rather than becoming empty.
        Self(if trimmed.is_empty() { text } else { trimmed }.to_owned())
    }
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

/// How much of a [`Clipped`] value is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ClipMode {
    /// Shown in full.
    Full,
    /// Clamped by the CSS to this many lines; the full text is still in the DOM.
    Lines(u8),
    /// Shortened to this many characters, with an ellipsis.
    Chars(usize),
}

/// Text that may be shown shortened. Holding the full value is not optional, so a
/// component can always declare `data-fc-truncated` and keep the full value reachable.
///
/// ```
/// use fc_dioxus::prelude::Clipped;
/// let id = Clipped::chars("9f2c1e04b7a3", 7);
/// assert_eq!(id.shown(), "9f2c1e0…");
/// assert!(id.is_truncated());
/// assert!(!Clipped::chars("short", 7).is_truncated());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Clipped {
    full: String,
    mode: ClipMode,
}
impl Clipped {
    /// Text shown in full.
    #[must_use]
    pub fn full(text: impl Into<String>) -> Self {
        Self {
            full: text.into(),
            mode: ClipMode::Full,
        }
    }
    /// Text the CSS clamps to `lines`. It may or may not overflow, so it is declared
    /// truncated whenever it is clamped at all.
    #[must_use]
    pub fn lines(text: impl Into<String>, lines: u8) -> Self {
        Self {
            full: text.into(),
            mode: ClipMode::Lines(lines),
        }
    }
    /// Text shortened to at most `chars` characters, never splitting one.
    #[must_use]
    pub fn chars(text: impl Into<String>, chars: usize) -> Self {
        Self {
            full: text.into(),
            mode: ClipMode::Chars(chars),
        }
    }
    /// The complete value.
    #[must_use]
    pub fn full_text(&self) -> &str {
        &self.full
    }
    /// The text to display.
    #[must_use]
    pub fn shown(&self) -> String {
        match self.mode {
            ClipMode::Chars(limit) if self.full.chars().count() > limit => {
                let mut shown: String = self.full.chars().take(limit).collect();
                shown.push('…');
                shown
            }
            _ => self.full.clone(),
        }
    }
    /// Whether the displayed text may be less than the full value.
    #[must_use]
    pub fn is_truncated(&self) -> bool {
        match self.mode {
            ClipMode::Full => false,
            ClipMode::Lines(_) => true,
            ClipMode::Chars(limit) => self.full.chars().count() > limit,
        }
    }
}

/// Whether a literal has no visible text, for [`name!`](crate::name)'s compile-time check.
#[doc(hidden)]
#[must_use]
pub const fn is_blank(text: &str) -> bool {
    let mut rest = text.as_bytes();
    while let [first, tail @ ..] = rest {
        if !first.is_ascii_whitespace() {
            return false;
        }
        rest = tail;
    }
    true
}

/// An [`AccessibleName`] from a string literal, checked when the program compiles.
///
/// ```
/// use fc_dioxus::{name, prelude::*};
/// let close: AccessibleName = name!("Close");
/// assert_eq!(close.as_str(), "Close");
/// ```
///
/// A blank literal does not compile:
///
/// ```compile_fail
/// let _ = fc_dioxus::name!("  ");
/// ```
#[macro_export]
macro_rules! name {
    ($text:literal) => {{
        const _: () = assert!(
            !$crate::core::text::is_blank($text),
            "accessible names must contain text"
        );
        $crate::core::text::AccessibleName::from_checked_literal($text)
    }};
}

//! Validated identifiers and deterministic, root-scoped generated IDs.

use dioxus::core::current_scope_id;
use dioxus::prelude::*;
use std::{cell::Cell, fmt, rc::Rc};

/// A non-empty HTML identifier without whitespace or control characters.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Id(String);
impl Id {
    /// The identifier, suitable for an ARIA reference.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl TryFrom<&str> for Id {
    type Error = InvalidId;
    /// # Errors
    /// Rejects empty identifiers, whitespace, and control characters.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.is_empty() || value.chars().any(|c| c.is_whitespace() || c.is_control()) {
            Err(InvalidId)
        } else {
            Ok(Self(value.to_owned()))
        }
    }
}
impl TryFrom<String> for Id {
    type Error = InvalidId;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}
impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
/// The input cannot be an HTML identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidId;
impl fmt::Display for InvalidId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("id must be non-empty without whitespace or controls")
    }
}
impl std::error::Error for InvalidId {}

#[derive(Clone)]
pub(crate) struct IdAllocator {
    prefix: usize,
    next: Rc<Cell<u64>>,
}
impl IdAllocator {
    pub(crate) fn new() -> Self {
        Self {
            prefix: current_scope_id().0,
            next: Rc::new(Cell::new(0)),
        }
    }
    fn next(&self) -> Id {
        let next = self.next.get();
        self.next.set(next.saturating_add(1));
        Id(format!("fc-{}-{next}", self.prefix))
    }
}

/// Allocate once per hook, in render order. Identical trees produce identical IDs.
///
/// Prefer a `FutureCss` root. Outside one, IDs are scoped to the current Dioxus
/// component and hook order rather than a process-global counter.
#[must_use]
pub fn use_id() -> Id {
    let allocator = use_hook(|| {
        try_consume_context::<IdAllocator>().unwrap_or_else(|| provide_context(IdAllocator::new()))
    });
    use_hook(|| allocator.next())
}

//! State shared across components, without inventing new upstream values.

use super::id::Id;
use dioxus::prelude::EventHandler;

/// The orientation of a component.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Orientation {
    /// A horizontal separator or row.
    #[default]
    Horizontal,
    /// A vertical separator or column.
    Vertical,
}

/// Whether a control can be activated, and whether it remains discoverable.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Availability {
    /// The action is available.
    #[default]
    Enabled,
    /// Native disabled state, removed from sequential focus.
    Disabled,
    /// Focusable but inactive; visible explanatory text is referenced by ID.
    Unavailable {
        /// ID of the element explaining the unavailable action.
        reason: Id,
    },
}
impl Availability {
    /// Drop activation handlers for both forms of unavailability.
    #[must_use]
    pub fn guard<T: 'static>(&self, handler: Option<EventHandler<T>>) -> Option<EventHandler<T>> {
        if *self == Self::Enabled {
            handler
        } else {
            None
        }
    }
}

/// Upstream's lifecycle vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Lifecycle {
    /// Not yet scheduled.
    Pending,
    /// Waiting to execute.
    Queued,
    /// Incremental output is arriving.
    Streaming,
    /// Work is executing.
    Running,
    /// Work completed.
    Complete,
    /// Work was cancelled.
    Aborted,
    /// Permission was denied.
    Denied,
    /// Work failed.
    Error,
}
impl Lifecycle {
    /// The upstream attribute value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Queued => "queued",
            Self::Streaming => "streaming",
            Self::Running => "running",
            Self::Complete => "complete",
            Self::Aborted => "aborted",
            Self::Denied => "denied",
            Self::Error => "error",
        }
    }
}

/// The author role in upstream's message contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MessageRole {
    /// User-authored message.
    User,
    /// Assistant-authored message.
    Assistant,
    /// System message.
    System,
    /// Tool result.
    Tool,
}
impl MessageRole {
    /// The upstream attribute value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::System => "system",
            Self::Tool => "tool",
        }
    }
}

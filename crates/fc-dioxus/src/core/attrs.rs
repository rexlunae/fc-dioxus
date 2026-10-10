//! Typed attributes for upstream's dual-audience contract.

use super::{
    id::Id,
    state::{Lifecycle, MessageRole},
};
use dioxus::prelude::*;

/// A timestamp for `data-fc-ts`: epoch milliseconds, or a validated ISO-8601 UTC instant.
/// Arbitrary strings cannot enter the DOM contract.
///
/// ```
/// use fc_dioxus::prelude::Timestamp;
/// assert!(Timestamp::iso8601("2026-10-09T12:00:00Z").is_ok());
/// assert!(Timestamp::iso8601("2026-10-09T12:00:00.250Z").is_ok());
/// assert!(Timestamp::iso8601("yesterday").is_err());
/// assert!(Timestamp::iso8601("2026-13-09T12:00:00Z").is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Timestamp(TimestampRepr);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum TimestampRepr {
    Millis(u64),
    Iso(String),
}
impl Timestamp {
    /// Construct an epoch-millisecond timestamp.
    #[must_use]
    pub const fn from_epoch_millis(value: u64) -> Self {
        Self(TimestampRepr::Millis(value))
    }
    /// Accept `YYYY-MM-DDTHH:MM:SS[.fraction]Z` with in-range fields.
    ///
    /// # Errors
    /// Returns `InvalidTimestamp` for anything else, including offsets other than `Z`.
    pub fn iso8601(value: &str) -> Result<Self, InvalidTimestamp> {
        let bytes = value.as_bytes();
        let digits = |range: std::ops::Range<usize>| -> Option<u32> {
            let part = value.get(range)?;
            if part.bytes().all(|b| b.is_ascii_digit()) {
                part.parse().ok()
            } else {
                None
            }
        };
        let separators = [(4, b'-'), (7, b'-'), (10, b'T'), (13, b':'), (16, b':')];
        let shape = bytes.len() >= 20
            && bytes.last() == Some(&b'Z')
            && separators
                .iter()
                .all(|(index, byte)| bytes.get(*index) == Some(byte));
        let fraction = value.get(19..value.len().saturating_sub(1)).unwrap_or("");
        let fraction_ok = fraction.is_empty()
            || (fraction.len() > 1
                && fraction.starts_with('.')
                && fraction.bytes().skip(1).all(|b| b.is_ascii_digit()));
        let fields = (
            digits(0..4),
            digits(5..7),
            digits(8..10),
            digits(11..13),
            digits(14..16),
            digits(17..19),
        );
        match fields {
            (Some(_), Some(month), Some(day), Some(hour), Some(minute), Some(second))
                if shape
                    && fraction_ok
                    && (1..=12).contains(&month)
                    && (1..=31).contains(&day)
                    && hour < 24
                    && minute < 60
                    && second < 61 =>
            {
                Ok(Self(TimestampRepr::Iso(value.to_owned())))
            }
            _ => Err(InvalidTimestamp),
        }
    }
}
impl std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            TimestampRepr::Millis(value) => write!(f, "{value}"),
            TimestampRepr::Iso(value) => f.write_str(value),
        }
    }
}
/// The text is not an ISO-8601 UTC instant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidTimestamp;
impl std::fmt::Display for InvalidTimestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("expected an ISO-8601 UTC timestamp such as 2026-10-09T12:00:00Z")
    }
}
impl std::error::Error for InvalidTimestamp {}

/// Only component kinds defined upstream can be emitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Kind {
    /// A chat surface.
    Chat,
    /// A message.
    Message,
    /// A tool call.
    ToolCall,
    /// A reasoning trace.
    Reasoning,
    /// An agent's live status row.
    AgentStatus,
    /// A request for the user's approval.
    Approval,
    /// A device authorization code.
    DeviceAuth,
    /// A dialog.
    Dialog,
    /// A metric.
    Metric,
    /// A toast notification.
    Toast,
    /// A file transfer.
    Transfer,
    /// A question put to the user.
    UserPrompt,
}
impl Kind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Chat => "chat",
            Self::Message => "message",
            Self::ToolCall => "tool-call",
            Self::Reasoning => "reasoning",
            Self::AgentStatus => "agent-status",
            Self::Approval => "approval",
            Self::DeviceAuth => "device-auth",
            Self::Dialog => "dialog",
            Self::Metric => "metric",
            Self::Toast => "toast",
            Self::Transfer => "transfer",
            Self::UserPrompt => "user-prompt",
        }
    }
}

/// Machine-readable state. There are no free-form payload fields.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FcData {
    /// Component kind.
    pub kind: Option<Kind>,
    /// A stable runtime identifier; never use credentials as identifiers.
    pub id: Option<Id>,
    /// Monotonic ordering integer.
    pub seq: Option<u64>,
    /// Message author role.
    pub role: Option<MessageRole>,
    /// Lifecycle state.
    pub state: Option<Lifecycle>,
    /// Epoch-millisecond timestamp.
    pub ts: Option<Timestamp>,
}
impl FcData {
    /// Render only present values; absence is not serialized as an empty string.
    #[must_use]
    pub fn attributes(&self) -> Vec<Attribute> {
        let mut attrs = Vec::new();
        for (name, value) in [
            ("data-fc-kind", self.kind.map(|v| v.as_str().to_owned())),
            ("data-fc-id", self.id.as_ref().map(ToString::to_string)),
            ("data-fc-seq", self.seq.map(|v| v.to_string())),
            ("data-fc-role", self.role.map(|v| v.as_str().to_owned())),
            ("data-fc-state", self.state.map(|v| v.as_str().to_owned())),
            ("data-fc-ts", self.ts.as_ref().map(ToString::to_string)),
        ] {
            if let Some(value) = value {
                attrs.push(Attribute::new(name, value, None, false));
            }
        }
        attrs
    }
}

pub(crate) fn theme_attributes(
    theme: Option<&'static str>,
    brand: Option<&'static str>,
    design: Option<&'static str>,
    palette: Option<&'static str>,
) -> Vec<Attribute> {
    let mut attrs = vec![Attribute::new(
        "data-fc-version",
        super::assets::CSS_VERSION,
        None,
        false,
    )];
    if let Some(theme) = theme {
        attrs.push(Attribute::new("data-fc-theme", theme, None, false));
    }
    if let Some(brand) = brand {
        attrs.push(Attribute::new("data-fc-brand", brand, None, false));
    }
    if let Some(design) = design {
        attrs.push(Attribute::new("data-fc-design", design, None, false));
    }
    if let Some(palette) = palette {
        attrs.push(Attribute::new("data-fc-palette", palette, None, false));
    }
    attrs
}

// Consumer classes are additive; native semantics and state remain component-owned.
pub(crate) fn component_attributes(
    attributes: Vec<Attribute>,
    mut classes: super::class::ClassList,
    protected: &[&str],
) -> Vec<Attribute> {
    for attr in &attributes {
        if attr.name == "class" {
            if let dioxus::core::AttributeValue::Text(value) = &attr.value {
                classes = classes.with(value);
            }
        }
    }
    let mut attributes: Vec<_> = attributes
        .into_iter()
        .filter(|attr| attr.name != "class" && !protected.contains(&attr.name))
        .collect();
    attributes.push(Attribute::new("class", classes.to_string(), None, false));
    attributes
}

pub(crate) fn availability_attributes(availability: &super::state::Availability) -> Vec<Attribute> {
    use super::state::Availability;
    match availability {
        Availability::Enabled => Vec::new(),
        Availability::Disabled => vec![Attribute::new("disabled", true, None, false)],
        Availability::Unavailable { reason } => vec![
            Attribute::new("aria-disabled", "true", None, false),
            Attribute::new("aria-describedby", reason.to_string(), None, false),
        ],
    }
}

/// Declare truncation and keep the full value reachable, as upstream's contract requires.
pub(crate) fn clipped_attributes(text: &super::text::Clipped) -> Vec<Attribute> {
    let mut attributes = truncated_attribute(text);
    attributes.extend(full_text_title(text));
    attributes
}

/// `data-fc-truncated`, for the element that represents the whole record.
pub(crate) fn truncated_attribute(text: &super::text::Clipped) -> Vec<Attribute> {
    if text.is_truncated() {
        vec![Attribute::new("data-fc-truncated", "true", None, false)]
    } else {
        Vec::new()
    }
}

/// The full value as a `title`, for the element showing the shortened text.
pub(crate) fn full_text_title(text: &super::text::Clipped) -> Vec<Attribute> {
    if text.is_truncated() {
        vec![Attribute::new(
            "title",
            text.full_text().to_owned(),
            None,
            false,
        )]
    } else {
        Vec::new()
    }
}

/// Hold a disclosure-like region open or closed regardless of focus; absent leaves it to
/// the CSS (`data-fc-open`).
pub(crate) fn open_attribute(open: Option<bool>) -> Vec<Attribute> {
    open.map(|open| Attribute::new("data-fc-open", open.to_string(), None, false))
        .into_iter()
        .collect()
}

/// Mark an element that is animating out before removal (`data-fc-leaving`).
pub(crate) fn leaving_attribute(leaving: bool) -> Vec<Attribute> {
    if leaving {
        vec![Attribute::new("data-fc-leaving", "true", None, false)]
    } else {
        Vec::new()
    }
}

/// A tooltip dismissed with Escape until the pointer or focus leaves (`data-fc-dismissed`).
pub(crate) fn dismissed_attribute(dismissed: bool) -> Vec<Attribute> {
    if dismissed {
        vec![Attribute::new("data-fc-dismissed", "true", None, false)]
    } else {
        Vec::new()
    }
}

/// Which way a carousel control pages (`data-fc-carousel`).
pub(crate) fn carousel_control_attribute(next: bool) -> Attribute {
    Attribute::new(
        "data-fc-carousel",
        if next { "next" } else { "prev" },
        None,
        false,
    )
}

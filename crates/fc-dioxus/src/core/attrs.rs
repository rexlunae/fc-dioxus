//! Typed attributes for upstream's dual-audience contract.

use super::{
    id::Id,
    state::{Lifecycle, MessageRole},
};
use dioxus::prelude::*;

/// An epoch-millisecond timestamp; arbitrary strings cannot enter the DOM contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Timestamp(u64);
impl Timestamp {
    /// Construct an epoch-millisecond timestamp.
    #[must_use]
    pub const fn from_epoch_millis(value: u64) -> Self {
        Self(value)
    }
}

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
}
impl Kind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Chat => "chat",
            Self::Message => "message",
            Self::ToolCall => "tool-call",
            Self::Reasoning => "reasoning",
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
            ("data-fc-ts", self.ts.map(|v| v.0.to_string())),
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
    if text.is_truncated() {
        vec![
            Attribute::new("data-fc-truncated", "true", None, false),
            Attribute::new("title", text.full_text().to_owned(), None, false),
        ]
    } else {
        Vec::new()
    }
}

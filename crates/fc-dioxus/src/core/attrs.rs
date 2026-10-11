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
    /// A multiple-choice picker.
    Multiselect,
    /// The theme picker.
    ThemePicker,
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
            Self::Multiselect => "multiselect",
            Self::ThemePicker => "theme-picker",
        }
    }
}

/// Attributes headed for an rsx spread (`..list`), kept sorted by name on the way in.
///
/// Dioxus diffs a spread as a merge of two name-sorted lists, but passes a spread
/// through as given. An unsorted list mis-diffs whenever its membership changes: an
/// icon button that became enabled lost `aria-label` along with `disabled`, because
/// the merge walked past it. rsx calls `into_boxed_slice` on every spread, so sorting
/// there covers every list built from this type.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Spread(Vec<Attribute>);
impl Spread {
    /// The list as rsx consumes it: sorted by name, stably, so a later duplicate still
    /// follows an earlier one.
    #[must_use]
    pub fn into_boxed_slice(mut self) -> Box<[Attribute]> {
        self.0.sort_by(|a, b| a.name.cmp(b.name));
        self.0.into_boxed_slice()
    }
}
impl std::ops::Deref for Spread {
    type Target = Vec<Attribute>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for Spread {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl From<Vec<Attribute>> for Spread {
    fn from(list: Vec<Attribute>) -> Self {
        Self(list)
    }
}
impl From<Spread> for Vec<Attribute> {
    fn from(list: Spread) -> Self {
        list.0
    }
}
impl FromIterator<Attribute> for Spread {
    fn from_iter<I: IntoIterator<Item = Attribute>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}
impl Extend<Attribute> for Spread {
    fn extend<I: IntoIterator<Item = Attribute>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}
impl IntoIterator for Spread {
    type Item = Attribute;
    type IntoIter = std::vec::IntoIter<Attribute>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// A [`Spread`] from attributes, as `vec!` makes a `Vec`.
#[macro_export]
macro_rules! spread {
    ($($attribute:expr),* $(,)?) => {
        $crate::core::attrs::Spread::from(::std::vec![$($attribute),*])
    };
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
    pub fn attributes(&self) -> Spread {
        let mut attrs = Spread::default();
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

/// A root's theme attributes by name, absent ones included so a mirror can clear them.
pub(crate) type ThemePairs = [(&'static str, Option<&'static str>); 4];

pub(crate) fn theme_pairs(
    theme: Option<&'static str>,
    brand: Option<&'static str>,
    design: Option<&'static str>,
    palette: Option<&'static str>,
) -> ThemePairs {
    [
        ("data-fc-theme", theme),
        ("data-fc-brand", brand),
        ("data-fc-design", design),
        ("data-fc-palette", palette),
    ]
}

pub(crate) fn theme_attributes(pairs: &ThemePairs) -> Spread {
    let mut attrs = crate::spread![Attribute::new(
        "data-fc-version",
        super::assets::CSS_VERSION,
        None,
        false,
    )];
    for (name, value) in pairs {
        if let Some(value) = value {
            attrs.push(Attribute::new(name, *value, None, false));
        }
    }
    attrs
}

// Consumer classes are additive; native semantics and state remain component-owned.
pub(crate) fn component_attributes(
    attributes: Vec<Attribute>,
    mut classes: super::class::ClassList,
    protected: &[&str],
) -> Spread {
    for attr in &attributes {
        if attr.name == "class" {
            if let dioxus::core::AttributeValue::Text(value) = &attr.value {
                classes = classes.with(value);
            }
        }
    }
    let mut attributes: Spread = attributes
        .into_iter()
        .filter(|attr| attr.name != "class" && !protected.contains(&attr.name))
        .collect();
    attributes.push(Attribute::new("class", classes.to_string(), None, false));
    attributes
}

pub(crate) fn availability_attributes(availability: &super::state::Availability) -> Spread {
    use super::state::Availability;
    match availability {
        Availability::Enabled => Spread::default(),
        Availability::Disabled => crate::spread![Attribute::new("disabled", true, None, false)],
        Availability::Unavailable { reason } => crate::spread![
            Attribute::new("aria-disabled", "true", None, false),
            Attribute::new("aria-describedby", reason.to_string(), None, false),
        ],
    }
}

/// Declare truncation and keep the full value reachable, as upstream's contract requires.
pub(crate) fn clipped_attributes(text: &super::text::Clipped) -> Spread {
    let mut attributes = truncated_attribute(text);
    attributes.extend(full_text_title(text));
    attributes
}

/// `data-fc-truncated`, for the element that represents the whole record.
pub(crate) fn truncated_attribute(text: &super::text::Clipped) -> Spread {
    if text.is_truncated() {
        crate::spread![Attribute::new("data-fc-truncated", "true", None, false)]
    } else {
        Spread::default()
    }
}

/// The full value as a `title`, for the element showing the shortened text.
pub(crate) fn full_text_title(text: &super::text::Clipped) -> Spread {
    if text.is_truncated() {
        crate::spread![Attribute::new(
            "title",
            text.full_text().to_owned(),
            None,
            false,
        )]
    } else {
        Spread::default()
    }
}

/// Hold a disclosure-like region open or closed regardless of focus; absent leaves it to
/// the CSS (`data-fc-open`).
pub(crate) fn open_attribute(open: Option<bool>) -> Spread {
    open.map(|open| Attribute::new("data-fc-open", open.to_string(), None, false))
        .into_iter()
        .collect()
}

/// Mark an element that is animating out before removal (`data-fc-leaving`).
pub(crate) fn leaving_attribute(leaving: bool) -> Spread {
    if leaving {
        crate::spread![Attribute::new("data-fc-leaving", "true", None, false)]
    } else {
        Spread::default()
    }
}

/// A tooltip dismissed with Escape until the pointer or focus leaves (`data-fc-dismissed`).
pub(crate) fn dismissed_attribute(dismissed: bool) -> Spread {
    if dismissed {
        crate::spread![Attribute::new("data-fc-dismissed", "true", None, false)]
    } else {
        Spread::default()
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

/// Markers upstream's multiselect markup uses to find its parts (`data-fc-*` flags).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MultiselectPart {
    /// The live summary of what is chosen.
    Summary,
    /// The filter input.
    Filter,
    /// The "no matches" message.
    Empty,
    /// "Select visible".
    SelectAll,
    /// "Clear".
    Clear,
}
pub(crate) fn multiselect_part(part: MultiselectPart) -> Attribute {
    match part {
        MultiselectPart::Summary => Attribute::new("data-fc-selection-summary", "", None, false),
        MultiselectPart::Filter => Attribute::new("data-fc-filter", "", None, false),
        MultiselectPart::Empty => Attribute::new("data-fc-empty", "", None, false),
        MultiselectPart::SelectAll => {
            Attribute::new("data-fc-selection-action", "all", None, false)
        }
        MultiselectPart::Clear => Attribute::new("data-fc-selection-action", "clear", None, false),
    }
}

/// Whether an option is chosen (`data-fc-selected`), and how many are (`data-fc-count`).
pub(crate) fn selected_attribute(selected: bool) -> Attribute {
    Attribute::new("data-fc-selected", selected.to_string(), None, false)
}
pub(crate) fn count_attribute(count: usize) -> Attribute {
    Attribute::new("data-fc-count", count.to_string(), None, false)
}

/// A date picker's parts: the day a button stands for (`data-fc-date`), which way a month
/// button pages (`data-fc-datepicker`), and the selection mirrored on the root
/// (`data-fc-value`). Dates are ISO strings, never free text.
pub(crate) fn date_attribute(iso: &str) -> Attribute {
    Attribute::new("data-fc-date", iso.to_owned(), None, false)
}
pub(crate) fn datepicker_nav_attribute(next: bool) -> Attribute {
    Attribute::new(
        "data-fc-datepicker",
        if next { "next" } else { "prev" },
        None,
        false,
    )
}
pub(crate) fn date_value_attribute(iso: Option<String>) -> Spread {
    iso.map(|iso| Attribute::new("data-fc-value", iso, None, false))
        .into_iter()
        .collect()
}

/// A select's machine-readable parts: the value an invoker or option stands for
/// (`data-fc-value`), the highlighted option (`data-fc-active`), and a placeholder shown
/// in place of a value (`data-fc-placeholder`). Values are app identifiers, not user text.
pub(crate) fn value_attribute(value: &str) -> Attribute {
    Attribute::new("data-fc-value", value.to_owned(), None, false)
}
pub(crate) fn active_attribute(active: bool) -> Spread {
    if active {
        crate::spread![Attribute::new("data-fc-active", "true", None, false)]
    } else {
        Spread::default()
    }
}
pub(crate) fn placeholder_attribute(placeholder: bool) -> Spread {
    if placeholder {
        crate::spread![Attribute::new("data-fc-placeholder", "true", None, false)]
    } else {
        Spread::default()
    }
}

/// The context strip's resolved state on its root (`data-fc-provider`, `-model`,
/// `-effort`), and whether a field applies (`data-fc-applicable`). Ids, not user text.
pub(crate) fn context_attributes(provider: &str, model: &str, effort: Option<&str>) -> Spread {
    let mut attrs = crate::spread![
        Attribute::new("data-fc-provider", provider.to_owned(), None, false),
        Attribute::new("data-fc-model", model.to_owned(), None, false),
    ];
    // Absent rather than empty: empty would claim effort is set to nothing.
    if let Some(effort) = effort {
        attrs.push(Attribute::new(
            "data-fc-effort",
            effort.to_owned(),
            None,
            false,
        ));
    }
    attrs
}
pub(crate) fn applicable_attribute(applicable: bool) -> Attribute {
    Attribute::new("data-fc-applicable", applicable.to_string(), None, false)
}

/// A composer's own lifecycle (`data-fc-state` of `idle`, `ready`, or `running`), and
/// whether a file is being dragged over it (`data-fc-dragging`).
pub(crate) fn composer_state_attribute(state: &'static str) -> Attribute {
    Attribute::new("data-fc-state", state, None, false)
}
pub(crate) fn dragging_attribute(dragging: bool) -> Spread {
    if dragging {
        crate::spread![Attribute::new("data-fc-dragging", "true", None, false)]
    } else {
        Spread::default()
    }
}
/// An item that failed (`data-fc-state="error"`), e.g. an attachment that could not be read.
pub(crate) fn error_state_attribute(error: bool) -> Spread {
    if error {
        crate::spread![Attribute::new("data-fc-state", "error", None, false)]
    } else {
        Spread::default()
    }
}

/// The language a code block holds (`data-fc-lang`), e.g. `rust`.
pub(crate) fn lang_attribute(lang: &str) -> Attribute {
    Attribute::new("data-fc-lang", lang.to_owned(), None, false)
}

/// A diff line's kind (`data-fc-diff` of `add` or `remove`); unchanged lines carry none.
pub(crate) fn diff_attribute(kind: &'static str) -> Attribute {
    Attribute::new("data-fc-diff", kind, None, false)
}

/// The git surface's machine-readable state. Paths and branch names are repository
/// data the application already shows; nothing here is secret.
#[derive(Debug, Clone, Copy)]
pub(crate) enum GitData {
    Branch,
    BranchState,
    Staged,
    Unstaged,
    Conflicted,
    Untracked,
    Empty,
    Path,
    Selected,
    Stage,
    State,
    Glyph,
    StagePath,
}
impl GitData {
    const fn name(self) -> &'static str {
        match self {
            Self::Branch => "data-fc-branch",
            Self::BranchState => "data-fc-branch-state",
            Self::Staged => "data-fc-staged",
            Self::Unstaged => "data-fc-unstaged",
            Self::Conflicted => "data-fc-conflicted",
            Self::Untracked => "data-fc-untracked",
            Self::Empty => "data-fc-empty",
            Self::Path => "data-fc-path",
            Self::Selected => "data-fc-selected",
            Self::Stage => "data-fc-stage",
            Self::State => "data-fc-state",
            Self::Glyph => "data-fc-glyph",
            Self::StagePath => "data-fc-stage-path",
        }
    }
    pub(crate) fn attribute(self, value: &(impl ToString + ?Sized)) -> Attribute {
        Attribute::new(self.name(), value.to_string(), None, false)
    }
}

/// The data grid's machine-readable state.
#[derive(Debug, Clone, Copy)]
pub(crate) enum GridData {
    /// The cell typing goes to.
    Active,
    /// Edited and not yet saved.
    Dirty,
    /// The true row count, which a virtualized grid renders only part of.
    RowCount,
    /// The true column count.
    ColCount,
}
impl GridData {
    const fn name(self) -> &'static str {
        match self {
            Self::Active => "data-fc-active",
            Self::Dirty => "data-fc-dirty",
            Self::RowCount => "data-fc-rowcount",
            Self::ColCount => "data-fc-colcount",
        }
    }
    pub(crate) fn attribute(self, value: &(impl ToString + ?Sized)) -> Attribute {
        Attribute::new(self.name(), value.to_string(), None, false)
    }
}

/// `data-fc-state` in a component's own upstream vocabulary rather than the shared
/// lifecycle: a terminal's `connected` and `disconnected`, a webview's `idle`,
/// `loading`, `loaded`, and `error`, or an address bar's `insecure`.
pub(crate) fn named_state_attribute(value: &'static str) -> Attribute {
    Attribute::new("data-fc-state", value, None, false)
}

/// The theme picker's selections, mirrored so automation can read what is chosen without
/// parsing labels.
#[derive(Debug, Clone, Copy)]
pub(crate) enum ThemePickerData {
    Design,
    Palette,
    Scheme,
    Contrast,
}
impl ThemePickerData {
    const fn name(self) -> &'static str {
        match self {
            Self::Design => "data-fc-design-selection",
            Self::Palette => "data-fc-palette-selection",
            Self::Scheme => "data-fc-scheme-selection",
            Self::Contrast => "data-fc-contrast-selection",
        }
    }
    pub(crate) fn attribute(self, value: &'static str) -> Attribute {
        Attribute::new(self.name(), value, None, false)
    }
}

/// A formatting control's command, read by upstream's editor binding.
pub(crate) fn editor_mark_attribute(mark: &'static str) -> Attribute {
    Attribute::new("data-fc-mark", mark, None, false)
}
/// A block control's command, read by upstream's editor binding.
pub(crate) fn editor_block_attribute(block: &'static str) -> Attribute {
    Attribute::new("data-fc-block", block, None, false)
}

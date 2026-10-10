//! Components implemented against the pinned upstream CSS and fixtures.

pub mod divider;

pub mod alert;
pub mod avatar;
pub mod badge;
pub mod breadcrumb;
pub mod button;
pub mod button_group;
pub mod card;
pub mod check;
pub mod description_list;
pub mod disclosure;
pub mod empty_state;
pub mod fab;
pub mod field;
pub mod list;
pub mod loading;
pub mod segmented;
pub mod spinner;
pub mod split_button;

use crate::core::state::HeadingLevel;
use dioxus::prelude::*;

/// The part class a group gives the controls inside it, so `ButtonGroup { Button {..} }`
/// needs no class from the author. A nested group replaces it for its own children.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GroupItem {
    /// Class for every item.
    pub(crate) item: String,
    /// Extra class for a text control, which a joined group sizes to fill.
    pub(crate) control: Option<String>,
}

pub(crate) fn provide_group_item(item: String, control: Option<String>) {
    use_context_provider(|| GroupItem { item, control });
}

/// The item class from the nearest group, for a button-like control.
pub(crate) fn group_item_class() -> Option<String> {
    try_consume_context::<GroupItem>().map(|group| group.item)
}

/// The item and control classes from the nearest group, for a text control.
pub(crate) fn group_control_class() -> Option<String> {
    try_consume_context::<GroupItem>().map(|group| match group.control {
        Some(control) => format!("{} {control}", group.item),
        None => group.item,
    })
}

/// A heading at a caller-chosen level, so components never hard-code document outline.
pub(crate) fn heading(level: HeadingLevel, class: String, children: Element) -> Element {
    match level {
        HeadingLevel::H1 => rsx! { h1 { class, {children} } },
        HeadingLevel::H2 => rsx! { h2 { class, {children} } },
        HeadingLevel::H3 => rsx! { h3 { class, {children} } },
        HeadingLevel::H4 => rsx! { h4 { class, {children} } },
        HeadingLevel::H5 => rsx! { h5 { class, {children} } },
        HeadingLevel::H6 => rsx! { h6 { class, {children} } },
    }
}

//! Components implemented against the pinned upstream CSS and fixtures.

pub mod divider;

pub mod alert;
pub mod avatar;
pub mod badge;
pub mod breadcrumb;
pub mod button;
pub mod card;
pub mod check;
pub mod description_list;
pub mod disclosure;
pub mod empty_state;
pub mod field;
pub mod list;
pub mod loading;
pub mod spinner;

use crate::core::state::HeadingLevel;
use dioxus::prelude::*;

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

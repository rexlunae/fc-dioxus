//! Compiled source examples, shared with the demo's visible snippets.

pub mod agent_status;
pub mod alert;
pub mod approval;
pub mod avatar;
pub mod badge;
pub mod breadcrumb;
pub mod button;
pub mod button_group;
pub mod card;
pub mod carousel;
pub mod chat;
pub mod check;
pub mod citation;
pub mod code;
pub mod composer;
pub mod context_strip;
pub mod control_group;
pub mod datepicker;
pub mod description_list;
pub mod device_auth;
pub mod dialog;
pub mod diff;
pub mod disclosure;
pub mod divider;
pub mod editor;
pub mod empty_state;
pub mod fab;
pub mod field;
pub mod future_css;
pub mod git;
pub mod grid;
pub mod icon;
pub mod input_group;
pub mod list;
pub mod loading;
pub mod masthead;
pub mod menu;
pub mod metric;
pub mod multiselect;
pub mod nav;
pub mod navbar;
pub mod pagination;
pub mod popover;
pub mod reasoning;
pub mod routing;
pub mod search;
pub mod segmented;
pub mod select;
pub mod slider;
pub mod spinner;
pub mod split_button;
pub mod splitter;
pub mod suggestion;
pub mod table;
pub mod tabs;
pub mod terminal;
pub mod theme_picker;
pub mod toast;
pub mod tool_call;
pub mod toolbar;
pub mod transfer;
pub mod user_prompt;
pub mod webview;

use fc_dioxus::prelude::{Id, Lifecycle, Record, Timestamp};

/// The runtime record the lifecycle examples mirror into `data-fc-*`.
pub(crate) fn record(kind: &str, state: Lifecycle, index: usize) -> Record {
    Record {
        id: Id::try_from(format!("{kind}-{}", index + 1)).ok(),
        seq: u64::try_from(index).ok(),
        state: Some(state),
        ts: Timestamp::iso8601("2026-10-09T12:00:00Z").ok(),
    }
}

/// A path long enough to test wrapping in every card.
pub(crate) fn long_path() -> String {
    format!(
        "workspace/{}README.md",
        "very-long-directory-name-".repeat(20)
    )
}

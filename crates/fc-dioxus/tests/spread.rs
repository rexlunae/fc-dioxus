//! Spread attributes survive membership changes.
//!
//! Dioxus diffs a spread as a merge of two name-sorted lists and does not sort it, so an
//! unsorted list drops the wrong attribute when one appears or disappears. This drives
//! the real diff and reads its DOM edits, which server rendering never shows.
#![forbid(unsafe_code)]

use dioxus::core::{AttributeValue, Mutation};
use fc_dioxus::prelude::*;
use std::cell::Cell;

thread_local! {
    static ENABLED: Cell<Option<Signal<bool>>> = const { Cell::new(None) };
}

#[component]
fn Toggled() -> Element {
    let enabled = use_signal(|| false);
    ENABLED.with(|cell| cell.set(Some(enabled)));
    rsx! {
        FutureCss { stylesheets: Stylesheets::External,
            Button { icon_label: name!("Back"),
                availability: if enabled() { Availability::Enabled } else { Availability::Disabled },
                "←"
            }
        }
    }
}

fn removed(edits: &[Mutation]) -> Vec<&'static str> {
    edits
        .iter()
        .filter_map(|edit| match edit {
            Mutation::SetAttribute {
                name,
                value: AttributeValue::None,
                ..
            } => Some(*name),
            _ => None,
        })
        .collect()
}

#[test]
fn enabling_an_icon_button_removes_disabled_and_keeps_its_name() {
    let mut dom = VirtualDom::new(Toggled);
    dom.rebuild_in_place();
    let stored = ENABLED.with(Cell::get);
    assert!(stored.is_some(), "the component stored its signal");
    let Some(mut enabled) = stored else { return };
    dom.in_runtime(|| enabled.set(true));
    let edits = dom.render_immediate_to_vec().edits;
    assert_eq!(
        removed(&edits),
        ["disabled"],
        "only `disabled` goes: {edits:?}"
    );
    dom.in_runtime(|| enabled.set(false));
    let edits = dom.render_immediate_to_vec().edits;
    assert!(
        removed(&edits).is_empty(),
        "disabling removes nothing: {edits:?}"
    );
}

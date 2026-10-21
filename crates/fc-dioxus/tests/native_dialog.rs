//! Dialog boolean presence on a host without browser boolean-attribute handling.
#![forbid(unsafe_code)]
#![cfg(not(target_arch = "wasm32"))]

use blitz_dom::Document;
use dioxus::prelude::*;
use dioxus_native_dom::{DioxusDocument, DocumentConfig};
use fc_dioxus::prelude::*;

#[component]
fn Sample(open: bool, modal: bool) -> Element {
    rsx! { FutureCss { stylesheets: Stylesheets::Inline,
        Dialog { open, modal, onclose: move |()| {},
            title: rsx! { "Review dialog" },
            p { "Dialog content" }
        }
    } }
}

#[test]
fn native_dialog_open_attribute_and_computed_visibility_match_state() {
    for (open, modal) in [(false, false), (true, false), (false, true), (true, true)] {
        let dom = VirtualDom::new_with_props(Sample, SampleProps { open, modal });
        let mut doc = DioxusDocument::new(dom, DocumentConfig::default());
        doc.initial_build();
        // Settle the unsupported-showModal fallback through the real native driver.
        while doc.poll(None) {}
        doc.inner.resolve(0.0);
        let node = doc
            .inner
            .query_selector("dialog")
            .ok()
            .flatten()
            .and_then(|id| doc.inner.get_node(id));
        let attribute = node.and_then(|node| node.element_data()).map(|element| {
            element
                .attrs()
                .iter()
                .any(|attr| attr.name.local.as_ref() == "open")
        });
        let hidden = node.and_then(|node| {
            node.primary_styles()
                .map(|style| style.clone_display().is_none())
        });
        assert_eq!(
            attribute,
            Some(open),
            "attribute: open={open}, modal={modal}"
        );
        assert_eq!(
            hidden,
            Some(!open),
            "visibility: open={open}, modal={modal}"
        );
    }
}

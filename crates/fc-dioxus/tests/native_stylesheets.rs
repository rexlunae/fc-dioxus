//! Stylesheet delivery on a native host without resource or HTML providers.
#![forbid(unsafe_code)]
#![cfg(not(target_arch = "wasm32"))]

use dioxus::prelude::*;
use dioxus_native_dom::{DioxusDocument, DocumentConfig};
use fc_dioxus::prelude::*;

#[component]
fn Sample(stylesheets: Stylesheets) -> Element {
    rsx! { FutureCss { stylesheets,
        div { id: "hidden", class: "fc-u-hidden", "Hidden by generated CSS" }
        div { id: "visible", "Visible control" }
    } }
}

#[test]
fn inline_generated_css_applies_without_host_resource_or_html_loading() {
    for (policy, hidden) in [
        (Stylesheets::Automatic, false),
        (Stylesheets::External, false),
        (Stylesheets::Inline, true),
    ] {
        let dom = VirtualDom::new_with_props(
            Sample,
            SampleProps {
                stylesheets: policy,
            },
        );
        let mut doc = DioxusDocument::new(dom, DocumentConfig::default());
        doc.initial_build();
        doc.inner.resolve(0.0);
        for (id, expected) in [("hidden", hidden), ("visible", false)] {
            let node = doc.inner.get_element_by_id(id).unwrap();
            let node = doc.inner.get_node(node).unwrap();
            let style = node.primary_styles().unwrap();
            assert_eq!(
                style.clone_display().is_none(),
                expected,
                "{policy:?}: {id}"
            );
        }
    }
}

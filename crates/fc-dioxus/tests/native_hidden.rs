//! HTML boolean presence on the supported native renderer.
#![forbid(unsafe_code)]
#![cfg(not(target_arch = "wasm32"))]

use dioxus::prelude::*;
use dioxus_native_dom::{DioxusDocument, DocumentConfig};
use fc_dioxus::prelude::*;

#[component]
fn Sample(size: f64) -> Element {
    rsx! { FutureCss { stylesheets: Stylesheets::Inline,
        Split { label: name!("Navigation width"), default_value: size,
            primary: rsx! { "Navigation" }, secondary: rsx! { "Content" },
        }
    } }
}

#[test]
fn expanded_split_omits_hidden_and_collapsed_split_keeps_it() {
    for (size, hidden) in [(18.0, false), (0.0, true)] {
        let dom = VirtualDom::new_with_props(Sample, SampleProps { size });
        let mut doc = DioxusDocument::new(dom, DocumentConfig::default());
        doc.initial_build();
        doc.inner.resolve(0.0);
        assert_hidden(&doc, ".fc-split__pane--primary", hidden);
    }
}

#[component]
fn Browser(failed: bool) -> Element {
    let src = if failed {
        "javascript:blocked"
    } else {
        "https://example.invalid/"
    };
    rsx! { FutureCss { stylesheets: Stylesheets::Inline,
        Webview { title: name!("Preview"), src }
    } }
}

#[test]
fn webview_errors_show_the_overlay_and_retry_control() {
    for failed in [false, true] {
        let dom = VirtualDom::new_with_props(Browser, BrowserProps { failed });
        let mut doc = DioxusDocument::new(dom, DocumentConfig::default());
        doc.initial_build();
        doc.inner.resolve(0.0);
        assert_hidden(&doc, ".fc-webview__overlay", !failed);
        let retry = doc
            .inner
            .query_selector(".fc-webview__overlay button")
            .ok()
            .flatten();
        assert_eq!(retry.is_some(), failed);
    }
}

fn assert_hidden(doc: &DioxusDocument, selector: &str, hidden: bool) {
    let node = doc
        .inner
        .query_selector(selector)
        .ok()
        .flatten()
        .and_then(|id| doc.inner.get_node(id));
    let attribute = node.and_then(|node| node.element_data()).map(|element| {
        element
            .attrs()
            .iter()
            .any(|attr| attr.name.local.as_ref() == "hidden")
    });
    let display = node.and_then(|node| {
        node.primary_styles()
            .map(|style| style.clone_display().is_none())
    });
    assert_eq!(attribute, Some(hidden), "hidden attribute: {selector}");
    assert_eq!(display, Some(hidden), "display: {selector}");
}

//! Compare component subtrees against the exact pinned upstream fixture markup.
#![forbid(unsafe_code)]

use fc_a11y::{Document, Node};
use fc_dioxus::prelude::*;
use std::collections::BTreeMap;

fn signature(node: &Node) -> (String, BTreeMap<String, String>) {
    let attrs = node
        .attributes
        .iter()
        .filter(|(name, _)| {
            name.as_str() == "class"
                || name.as_str() == "role"
                || name.starts_with("aria-")
                || name.starts_with("data-fc-")
        })
        .map(|(name, value)| {
            let normalized = if name == "class" {
                let mut classes: Vec<_> = value
                    .split_whitespace()
                    .filter(|c| c.starts_with("fc-"))
                    .collect();
                classes.sort_unstable();
                classes.join(" ")
            } else {
                value.clone()
            };
            (name.clone(), normalized)
        })
        .collect();
    (node.tag.clone(), attrs)
}

fn inside_fixture(document: &Document, node: &Node, fixture: &str) -> bool {
    let mut current = Some(node);
    while let Some(node) = current {
        if node.attr("data-fixture") == Some(fixture) {
            return true;
        }
        current = node.parent.and_then(|parent| document.nodes.get(parent));
    }
    false
}

#[test]
fn divider_markup_matches_every_upstream_reference_state() {
    let upstream = Document::parse(include_str!("../../upstream/fixtures.html"));
    let rendered = dioxus_ssr::render_element(rsx! {
        FutureCss { stylesheets: Stylesheets::External,
            fc_demo::examples::divider::Horizontal {}
            fc_demo::examples::divider::Vertical {}
        }
    });
    let ours = Document::parse(&rendered);
    let expected: Vec<_> = upstream
        .nodes
        .iter()
        .filter(|node| node.has_class("fc-divider") && inside_fixture(&upstream, node, "divider"))
        .map(signature)
        .collect();
    let actual: Vec<_> = ours
        .nodes
        .iter()
        .filter(|node| node.has_class("fc-divider"))
        .map(signature)
        .collect();
    assert_eq!(
        expected.len(),
        4,
        "upstream added a reference state; extend the demo"
    );
    assert_eq!(actual, expected);
}

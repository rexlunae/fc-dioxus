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

#[derive(Debug, PartialEq, Eq)]
struct Tree {
    tag: String,
    attrs: BTreeMap<String, String>,
    children: Vec<Self>,
}

fn tree(document: &Document, index: usize) -> Option<Tree> {
    tree_within(document, index, false)
}

fn tree_within(document: &Document, index: usize, hidden: bool) -> Option<Tree> {
    let node = document.nodes.get(index)?;
    let (tag, mut attrs) = signature(node);
    // Inside a hidden subtree, aria-hidden on a descendant changes nothing a reader
    // receives; a reusable component may repeat it, the reference markup need not.
    if hidden {
        attrs.remove("aria-hidden");
    }
    let hidden = hidden || node.attr("aria-hidden") == Some("true");
    // An input without a type is a text input; upstream writes it in some fixtures only.
    if tag == "input" && node.attr("type").is_none() {
        attrs.insert("type".to_owned(), "text".to_owned());
    }
    // IDs differ per render. The Dioxus API also requires explanatory text for
    // unavailable controls; fc-a11y separately verifies these added references.
    attrs.remove("aria-describedby");
    for name in ["type", "disabled", "checked"] {
        if let Some(value) = node.attr(name) {
            attrs.insert(
                name.to_owned(),
                if name == "type" {
                    value.to_owned()
                } else {
                    String::new()
                },
            );
        }
    }
    let children = document
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, child)| child.parent == Some(index))
        .filter_map(|(child, _)| tree_within(document, child, hidden))
        // A bare span around text is inert. Upstream wraps a busy button's label in one
        // in button's fixture and not in control-group's or split-button's.
        .filter(|child| {
            !(child.tag == "span" && child.attrs.is_empty() && child.children.is_empty())
        })
        .collect();
    Some(Tree {
        tag,
        attrs,
        children,
    })
}

type Case = (&'static str, &'static [&'static str], fn() -> Element);

/// Each fixture, the classes whose subtrees are compared, and the example rendering them.
const CASES: &[Case] = &[
    (
        "button",
        &["fc-btn"],
        || rsx! { fc_demo::examples::button::States {} },
    ),
    (
        "card",
        &["fc-card"],
        || rsx! { fc_demo::examples::card::States {} },
    ),
    (
        "badge",
        &["fc-badge", "fc-tag", "fc-kbd"],
        || rsx! { fc_demo::examples::badge::States {} },
    ),
    (
        "alert",
        &["fc-alert"],
        || rsx! { fc_demo::examples::alert::States {} },
    ),
    (
        "avatar",
        &["fc-avatar", "fc-avatar-group"],
        || rsx! { fc_demo::examples::avatar::States {} },
    ),
    (
        "spinner",
        &["fc-spinner", "fc-shimmer-text"],
        || rsx! { fc_demo::examples::spinner::States {} },
    ),
    (
        "loading",
        &["fc-skeleton", "fc-progress", "fc-progress-ring"],
        || rsx! { fc_demo::examples::loading::States {} },
    ),
    (
        "breadcrumb",
        &["fc-breadcrumb"],
        || rsx! { fc_demo::examples::breadcrumb::States {} },
    ),
    (
        "empty-state",
        &["fc-empty"],
        || rsx! { fc_demo::examples::empty_state::States {} },
    ),
    (
        "description-list",
        &["fc-dl"],
        || rsx! { fc_demo::examples::description_list::States {} },
    ),
    (
        "list",
        &["fc-list"],
        || rsx! { fc_demo::examples::list::States {} },
    ),
    (
        "check",
        &["fc-check", "fc-check-group", "fc-switch", "fc-switch-row"],
        || rsx! { fc_demo::examples::check::States {} },
    ),
    (
        "field",
        &["fc-field", "fc-input", "fc-input-group", "fc-error-summary"],
        || rsx! { fc_demo::examples::field::States {} },
    ),
    (
        "disclosure",
        &["fc-disclosure"],
        || rsx! { fc_demo::examples::disclosure::States {} },
    ),
    (
        "button-group",
        &["fc-button-group"],
        || rsx! { fc_demo::examples::button_group::States {} },
    ),
    (
        "control-group",
        &["fc-control-group"],
        || rsx! { fc_demo::examples::control_group::States {} },
    ),
    (
        "segmented",
        &["fc-segmented"],
        || rsx! { fc_demo::examples::segmented::States {} },
    ),
    (
        "split-button",
        &["fc-split-btn"],
        || rsx! { fc_demo::examples::split_button::States {} },
    ),
    (
        "fab",
        &["fc-fab", "fc-fab-menu"],
        || rsx! { fc_demo::examples::fab::States {} },
    ),
];

#[test]
fn styled_components_match_every_upstream_reference_subtree() {
    let upstream = Document::parse(include_str!("../../upstream/fixtures.html"));
    for &(fixture, classes, render) in CASES {
        let ours = Document::parse(&dioxus_ssr::render_element(
            rsx! { FutureCss { stylesheets: Stylesheets::External, {render()} } },
        ));
        let actual: Vec<_> = ours
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| classes.iter().any(|class| node.has_class(class)))
            .filter_map(|(index, _)| tree(&ours, index))
            .collect();
        let expected: Vec<_> = upstream
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| {
                classes.iter().any(|class| node.has_class(class))
                    && inside_fixture(&upstream, node, fixture)
            })
            .filter_map(|(index, _)| tree(&upstream, index))
            .collect();
        assert!(!expected.is_empty(), "missing upstream fixture {fixture}");
        for signature in expected {
            assert!(
                actual.contains(&signature),
                "{fixture} is missing upstream subtree {signature:#?}"
            );
        }
    }
}

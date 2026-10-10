//! SSR and static accessibility helpers for application tests.

use dioxus::prelude::*;

/// Render a component tree deterministically without requiring a browser.
#[must_use]
pub fn render_to_html(element: Element) -> String {
    dioxus_ssr::render_element(element)
}

/// Audit an HTML fragment using the currently implemented static rules.
/// This is not a claim of complete WCAG conformance; see the rule catalogue.
#[must_use]
pub fn audit(html: &str) -> fc_a11y::Report {
    fc_a11y::Audit.run(&fc_a11y::Document::parse(html))
}

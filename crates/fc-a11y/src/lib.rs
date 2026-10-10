//! Framework-independent static accessibility checks over an HTML tree.
//!
//! The initial catalogue checks image alternatives, ARIA references and their
//! uniqueness, and positive tabindex. It does not certify WCAG conformance.
#![forbid(unsafe_code)]

use std::collections::BTreeMap;

#[cfg(feature = "html")]
mod html;
pub mod rules;

/// An element in document order, independent of any parser or UI framework.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Node {
    /// Lowercase HTML tag name.
    pub tag: String,
    /// Attributes after HTML entity decoding.
    pub attributes: BTreeMap<String, String>,
    /// Index of the parent element, if any.
    pub parent: Option<usize>,
    /// Text directly inside this element.
    pub text: String,
}
impl Node {
    /// Return an attribute value without conflating missing and empty attributes.
    #[must_use]
    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attributes.get(name).map(String::as_str)
    }
    /// Whether this element has the given class token.
    #[must_use]
    pub fn has_class(&self, name: &str) -> bool {
        self.attr("class")
            .is_some_and(|value| value.split_whitespace().any(|class| class == name))
    }
}

/// A flattened HTML element tree. Indices always refer to document order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Document {
    /// The parsed elements.
    pub nodes: Vec<Node>,
}

/// The severity of a finding; uncertain or best-practice findings are warnings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Definite markup contract failure.
    Error,
    /// A pattern requiring review.
    Warning,
}

/// One actionable rule result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// Stable rule ID.
    pub rule: &'static str,
    /// Honest severity, without inferring layout from HTML.
    pub severity: Severity,
    /// WCAG success criteria relevant to the check.
    pub wcag: &'static [&'static str],
    /// Index of the offending element.
    pub node: usize,
    /// What failed.
    pub message: String,
    /// How to fix it.
    pub help: &'static str,
}

/// Results of the implemented rules, not a conformance certification.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    /// Findings in rule order and document order.
    pub findings: Vec<Finding>,
}
impl Report {
    /// Number of definite failures.
    #[must_use]
    pub fn errors(&self) -> usize {
        self.findings
            .iter()
            .filter(|finding| finding.severity == Severity::Error)
            .count()
    }
}

/// A stateless check over the parser-independent tree.
pub trait Rule: Send + Sync {
    /// Stable machine-readable ID.
    fn id(&self) -> &'static str;
    /// The check's relevant WCAG criteria.
    fn criteria(&self) -> &'static [&'static str];
    /// Append findings without mutating the document.
    fn check(&self, document: &Document, out: &mut Vec<Finding>);
}

/// Run the initial static rule catalogue.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Audit;
impl Audit {
    /// Audit a document or fragment; there are no page-only rules yet.
    #[must_use]
    pub fn run(self, document: &Document) -> Report {
        let mut report = Report::default();
        for rule in rules::catalogue() {
            rule.check(document, &mut report.findings);
        }
        report
    }
}

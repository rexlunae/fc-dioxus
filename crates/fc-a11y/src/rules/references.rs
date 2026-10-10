use crate::{Document, Finding, Rule, Severity};
use std::collections::{BTreeMap, BTreeSet};

const REFERENCES: &[&str] = &[
    "aria-labelledby",
    "aria-describedby",
    "aria-controls",
    "aria-activedescendant",
    "aria-owns",
    "aria-details",
    "aria-errormessage",
];

/// Require ARIA references and explicit label targets to exist (WCAG 1.3.1, 4.1.2).
#[derive(Debug, Clone, Copy, Default)]
pub struct IdRef;
impl Rule for IdRef {
    fn id(&self) -> &'static str {
        "idref"
    }
    fn criteria(&self) -> &'static [&'static str] {
        &["1.3.1", "4.1.2"]
    }
    fn check(&self, document: &Document, out: &mut Vec<Finding>) {
        let ids: BTreeSet<_> = document
            .nodes
            .iter()
            .filter_map(|node| node.attr("id"))
            .collect();
        for (index, node) in document.nodes.iter().enumerate() {
            let label = if node.tag == "label" {
                node.attr("for")
            } else {
                None
            };
            for target in REFERENCES
                .iter()
                .filter_map(|name| node.attr(name))
                .chain(label)
                .flat_map(str::split_whitespace)
            {
                if !ids.contains(target) {
                    out.push(Finding { rule: self.id(), severity: Severity::Error, wcag: self.criteria(), node: index,
                        message: format!("Reference to missing id {target:?}."), help: "Render the referenced element in the audited tree; fc-dioxus::use_id creates stable IDs for relationships." });
                }
            }
        }
    }
}

/// IDs used by accessible relationships must be unambiguous (WCAG 4.1.2).
#[derive(Debug, Clone, Copy, Default)]
pub struct DuplicateId;
impl Rule for DuplicateId {
    fn id(&self) -> &'static str {
        "duplicate-id"
    }
    fn criteria(&self) -> &'static [&'static str] {
        &["4.1.2"]
    }
    fn check(&self, document: &Document, out: &mut Vec<Finding>) {
        let referenced: BTreeSet<_> = document
            .nodes
            .iter()
            .flat_map(|node| {
                REFERENCES
                    .iter()
                    .filter_map(|name| node.attr(name))
                    .chain(if node.tag == "label" {
                        node.attr("for")
                    } else {
                        None
                    })
                    .flat_map(str::split_whitespace)
            })
            .collect();
        let mut counts = BTreeMap::new();
        for (index, node) in document.nodes.iter().enumerate() {
            if let Some(id) = node.attr("id").filter(|id| referenced.contains(id)) {
                let count = counts.entry(id).or_insert(0_u64);
                *count += 1;
                if *count > 1 {
                    out.push(Finding { rule: self.id(), severity: Severity::Error, wcag: self.criteria(), node: index,
                        message: format!("Referenced id {id:?} occurs more than once."), help: "Assign unique IDs to relationship targets; use fc-dioxus::use_id inside FutureCss." });
                }
            }
        }
    }
}

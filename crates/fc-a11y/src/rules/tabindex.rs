use crate::{Document, Finding, Rule, Severity};

/// Warn when positive tabindex changes document focus order (WCAG 2.4.3 review).
#[derive(Debug, Clone, Copy, Default)]
pub struct Tabindex;
impl Rule for Tabindex {
    fn id(&self) -> &'static str {
        "tabindex-positive"
    }
    fn criteria(&self) -> &'static [&'static str] {
        &["2.4.3"]
    }
    fn check(&self, document: &Document, out: &mut Vec<Finding>) {
        for (index, node) in document.nodes.iter().enumerate() {
            if node
                .attr("tabindex")
                .and_then(|value| value.trim().parse::<i64>().ok())
                .is_some_and(|value| value > 0)
            {
                out.push(Finding { rule: self.id(), severity: Severity::Warning, wcag: self.criteria(), node: index,
                    message: "Positive tabindex overrides document focus order.".to_owned(), help: "Use DOM order with tabindex=0 or native controls; use -1 for programmatic focus." });
            }
        }
    }
}

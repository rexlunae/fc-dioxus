use crate::{Document, Finding, Rule, Severity};

/// Require an explicit alternative for exposed HTML images (WCAG 1.1.1).
#[derive(Debug, Clone, Copy, Default)]
pub struct ImageAlt;
impl Rule for ImageAlt {
    fn id(&self) -> &'static str {
        "img-alt"
    }
    fn criteria(&self) -> &'static [&'static str] {
        &["1.1.1"]
    }
    fn check(&self, document: &Document, out: &mut Vec<Finding>) {
        for (index, node) in document.nodes.iter().enumerate() {
            if node.tag == "img"
                && node.attr("alt").is_none()
                && !hidden(document, index)
                && node
                    .attr("aria-label")
                    .is_none_or(|label| label.trim().is_empty())
                && node
                    .attr("aria-labelledby")
                    .is_none_or(|label| label.trim().is_empty())
                && node
                    .attr("title")
                    .is_none_or(|title| title.trim().is_empty())
            {
                out.push(Finding { rule: self.id(), severity: Severity::Error, wcag: self.criteria(), node: index,
                    message: "Image has no text alternative.".to_owned(), help: "Provide alt text, or alt=\"\" for decoration. fc-dioxus::Alt makes that choice explicit." });
            }
        }
    }
}

fn hidden(document: &Document, index: usize) -> bool {
    let mut current = Some(index);
    // A caller-built tree can contain invalid parent links. Bound traversal so
    // those links cannot hang an audit; parser-built trees always terminate early.
    for _ in 0..document.nodes.len() {
        let Some(node) = current.and_then(|index| document.nodes.get(index)) else {
            return false;
        };
        if node.attr("aria-hidden") == Some("true")
            || node.attr("hidden").is_some()
            || node.attr("inert").is_some()
        {
            return true;
        }
        current = node.parent;
    }
    false
}

use crate::{Document, Node};
use html5ever::{parse_document, tendril::TendrilSink};
use markup5ever_rcdom::{NodeData, RcDom};

impl Document {
    /// Parse HTML with the browser-compatible HTML5 tree builder.
    /// Missing document scaffolding is inserted just as a browser inserts it.
    #[must_use]
    pub fn parse(html: &str) -> Self {
        let dom = parse_document(RcDom::default(), html5ever::ParseOpts::default()).one(html);
        let mut document = Self::default();
        // RcDom's iterative destructor drains descendant lists, even when a
        // descendant still has another handle. Keep the root alive until traversal ends.
        let mut stack = vec![(dom.document.clone(), None)];
        while let Some((handle, parent)) = stack.pop() {
            let current = match &handle.data {
                NodeData::Element { name, attrs, .. } => {
                    let index = document.nodes.len();
                    document.nodes.push(Node {
                        tag: name.local.to_string(),
                        attributes: attrs
                            .borrow()
                            .iter()
                            .map(|attr| (attr.name.local.to_string(), attr.value.to_string()))
                            .collect(),
                        parent,
                        text: String::new(),
                    });
                    Some(index)
                }
                NodeData::Text { contents } => {
                    if let Some(node) = parent.and_then(|i| document.nodes.get_mut(i)) {
                        node.text.push_str(&contents.borrow());
                    }
                    parent
                }
                _ => parent,
            };
            for child in handle.children.borrow().iter().rev() {
                stack.push((child.clone(), current));
            }
        }
        document
    }
}

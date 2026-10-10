//! A collapsible reasoning trace, from reasoning.css.
use crate::{
    core::{
        assets,
        attrs::{component_attributes, FcData, Kind},
    },
    prelude::*,
    theme::use_stylesheets,
};
const BEM: Bem = Bem::new("fc-reasoning");

/// A model's reasoning, collapsed behind a summary line. The summary says "Thinking"
/// while the work is live and "Thought" once it has settled, unless `label` says
/// otherwise.
#[component]
pub fn Reasoning(
    state: Lifecycle,
    #[props(default)] label: Option<Element>,
    /// How long it took or has taken so far.
    #[props(default)]
    duration: Option<Element>,
    #[props(default)] open: bool,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let mut attributes = component_attributes(
        attributes,
        ClassList::new(BEM.block()).extend(class.as_deref()),
        &["open"],
    );
    attributes.extend(
        FcData {
            kind: Some(Kind::Reasoning),
            state: Some(state),
            ..FcData::default()
        }
        .attributes(),
    );
    let label = label.unwrap_or_else(|| {
        if state.is_settled() {
            rsx! { "Thought" }
        } else {
            rsx! { "Thinking" }
        }
    });
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_REASONING_CSS } }
        details { open, ..attributes,
            summary { class: BEM.element("summary"),
                span { class: BEM.element("label"), {label} }
                if let Some(duration) = duration { span { class: BEM.element("duration"), {duration} } }
            }
            div { class: BEM.element("body"), {children} }
        }
    }
}

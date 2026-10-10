//! What a region shows when it has nothing to show, from empty-state.css.
use crate::{
    core::{assets, attrs::component_attributes},
    prelude::*,
    theme::use_stylesheets,
};
const BEM: Bem = Bem::new("fc-empty");
fc_modifier! { enum EmptySize for BEM { Regular => None, Compact => "compact" } }
fc_modifier! { enum EmptyTone for BEM { Neutral => None, Error => "error" } }

/// An empty or failed region: an icon, a title, a sentence, and what to do next.
#[component]
pub fn EmptyState(
    #[props(default)] title: Option<Element>,
    #[props(default)] heading: HeadingLevel,
    /// Decorative; hidden from assistive technology.
    #[props(default)]
    icon: Option<Element>,
    #[props(default)] body: Option<Element>,
    #[props(default)] actions: Option<Element>,
    #[props(default)] compact: bool,
    #[props(default)] error: bool,
    /// Announce the error. Set it only when a user action led here, so a page that
    /// merely loads empty does not interrupt.
    #[props(default)]
    announce: bool,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let classes = ClassList::new(BEM.block())
        .with_modifier(
            BEM,
            if compact {
                EmptySize::Compact
            } else {
                EmptySize::Regular
            },
        )
        .with_modifier(
            BEM,
            if error {
                EmptyTone::Error
            } else {
                EmptyTone::Neutral
            },
        )
        .extend(class.as_deref());
    let mut attributes = component_attributes(attributes, classes, &["role"]);
    if error && announce {
        attributes.push(Attribute::new("role", "alert", None, false));
    }
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_EMPTY_STATE_CSS } }
        div { ..attributes,
            if let Some(icon) = icon { span { class: BEM.element("icon"), aria_hidden: "true", {icon} } }
            if let Some(title) = title { {super::heading(heading, BEM.element("title"), title)} }
            if let Some(body) = body { p { class: BEM.element("body"), {body} } }
            if let Some(actions) = actions { div { class: BEM.element("actions"), {actions} } }
        }
    }
}

//! Suggested prompts and follow-ups, from suggestion.css.
use crate::{
    core::{
        assets,
        attrs::{availability_attributes, component_attributes},
    },
    prelude::*,
    theme::use_stylesheets,
};
const LIST: Bem = Bem::new("fc-suggestions");
const ITEM: Bem = Bem::new("fc-suggestion");
fc_modifier! { enum ListLayout for LIST { Row => None, Grid => "grid" } }
fc_modifier! { enum ItemShape for ITEM { Card => None, Pill => "pill" } }

/// A named list of [`Suggestion`]s: cards in a grid, or a row of pills.
#[component]
pub fn Suggestions(
    label: AccessibleName,
    #[props(default)] grid: bool,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let attributes = component_attributes(
        attributes,
        ClassList::new(LIST.block())
            .with_modifier(
                LIST,
                if grid {
                    ListLayout::Grid
                } else {
                    ListLayout::Row
                },
            )
            .extend(class.as_deref()),
        &["aria-label"],
    );
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_SUGGESTION_CSS } }
        ul { aria_label: label.to_string(), ..attributes, {children} }
    }
}

/// One suggestion: a title and optional hint as a card, or plain text as a pill.
#[component]
pub fn Suggestion(
    #[props(default)] pill: bool,
    #[props(default)] hint: Option<Element>,
    #[props(default)] availability: Availability,
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    let class = ClassList::new(ITEM.block())
        .with_modifier(
            ITEM,
            if pill {
                ItemShape::Pill
            } else {
                ItemShape::Card
            },
        )
        .to_string();
    let onclick = availability.guard(onclick);
    let state = availability_attributes(&availability);
    rsx! {
        li {
            button { r#type: "button", class,
                onclick: move |event| if let Some(onclick) = onclick { onclick.call(event); },
                ..state,
                if pill { {children} } else {
                    span { class: ITEM.element("title"), {children} }
                    if let Some(hint) = hint { span { class: ITEM.element("hint"), {hint} } }
                }
            }
        }
    }
}

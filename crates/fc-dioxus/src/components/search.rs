//! A search landmark with a suggestions view, from search.css.
use crate::{
    core::{
        assets,
        attrs::{availability_attributes, component_attributes, open_attribute},
    },
    prelude::*,
    theme::use_stylesheets,
};
const BEM: Bem = Bem::new("fc-search");
// The input's and the view's modifiers hang off those elements, not the block.
const INPUT: Bem = Bem::new("fc-search__input");
const VIEW: Bem = Bem::new("fc-search__view");
fc_modifier! { enum ClearButton for INPUT { Native => None, Own => "own-clear" } }
fc_modifier! { enum ViewPosition for VIEW { Floating => None, Docked => "docked" } }

/// A search bar in a `role="search"` landmark, with an optional [`SearchView`].
///
/// With `onsubmit` it is a `<form>`, so Enter submits; without, it is a plain landmark
/// for filtering as you type.
#[component]
pub fn Search(
    label: AccessibleName,
    #[props(default)] value: Option<ReadSignal<String>>,
    #[props(default)] default_value: String,
    #[props(default)] oninput: Option<EventHandler<String>>,
    #[props(default)] onsubmit: Option<EventHandler<String>>,
    #[props(default)] placeholder: Option<String>,
    /// Trailing actions, e.g. voice search or a named "Clear search" button.
    #[props(default)]
    trailing: Option<Element>,
    /// The trailing slot holds its own clear button, so hide the browser's.
    #[props(default)]
    own_clear: bool,
    #[props(default)] view: Option<Element>,
    #[props(default)] invalid: bool,
    #[props(default)] availability: Availability,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let state = use_controllable(value, || default_value, oninput);
    let attributes = component_attributes(
        attributes,
        ClassList::new(BEM.block()).extend(class.as_deref()),
        &["role"],
    );
    let input_class = ClassList::new(INPUT.block())
        .with_modifier(
            INPUT,
            if own_clear {
                ClearButton::Own
            } else {
                ClearButton::Native
            },
        )
        .to_string();
    let input_state = availability_attributes(&availability);
    let body = rsx! {
        div { class: BEM.element("bar"),
            span { class: BEM.element("icon"), aria_hidden: "true", "⚲" }
            input { class: input_class, r#type: "search", aria_label: label.to_string(), placeholder,
                aria_invalid: invalid.then_some("true"), value: state.get(),
                oninput: move |event| state.set(event.value()), ..input_state }
            if let Some(trailing) = trailing { span { class: BEM.element("trailing"), {trailing} } }
        }
        {view}
    };
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_SEARCH_CSS } }
        if let Some(onsubmit) = onsubmit {
            form { role: "search",
                onsubmit: move |event| { event.prevent_default(); onsubmit.call(state.get()); },
                ..attributes, {body}
            }
        } else {
            div { role: "search", ..attributes, {body} }
        }
    }
}

/// Recent searches or suggestions under a [`Search`]. It shows while focus is in the
/// search; `open` holds it open or closed regardless.
#[component]
pub fn SearchView(
    label: AccessibleName,
    heading: Element,
    #[props(default)] docked: bool,
    #[props(default)] open: Option<bool>,
    /// Results are being fetched; the stale list stays readable.
    #[props(default)]
    busy: bool,
    children: Element,
) -> Element {
    let class = ClassList::new(VIEW.block())
        .with_modifier(
            VIEW,
            if docked {
                ViewPosition::Docked
            } else {
                ViewPosition::Floating
            },
        )
        .to_string();
    let open = open_attribute(open);
    rsx! {
        div { class, role: "region", aria_label: label.to_string(), aria_busy: busy.then_some("true"), ..open,
            p { class: BEM.element("view-label"), {heading} }
            {children}
        }
    }
}

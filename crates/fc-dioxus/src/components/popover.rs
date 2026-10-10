//! Native popovers and tooltips, from popover.css.
use crate::{
    core::{
        assets,
        attrs::{component_attributes, dismissed_attribute},
    },
    prelude::*,
    theme::use_stylesheets,
};
const POPOVER: Bem = Bem::new("fc-popover");
const TOOLTIP: Bem = Bem::new("fc-tooltip");
const BUBBLE: Bem = Bem::new("fc-tooltip__bubble");
fc_modifier! { enum BubbleSide for BUBBLE { Above => None, Below => "below" } }

/// Non-modal floating content on the native popover API. Open it from a button with
/// `popovertarget: "{id}"`; the platform supplies the top layer, light dismiss, Escape,
/// and focus return.
#[component]
pub fn Popover(
    id: Id,
    #[props(default)] title: Option<Element>,
    #[props(default)] footer: Option<Element>,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let mut attributes = component_attributes(
        attributes,
        ClassList::new(POPOVER.block()).extend(class.as_deref()),
        &["id", "popover"],
    );
    attributes.push(Attribute::new("popover", "auto", None, false));
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_POPOVER_CSS } }
        div { id: "{id}", ..attributes,
            if let Some(title) = title { p { class: POPOVER.element("title"), {title} } }
            {children}
            if let Some(footer) = footer { div { class: POPOVER.element("footer"), {footer} } }
        }
    }
}

/// The bubble a [`Button`] inside a [`Tooltip`] is described by.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DescribedBy(pub(crate) Id);

/// A short supplementary label shown on hover and keyboard focus.
///
/// The [`Button`] inside is described by the bubble automatically. Escape hides the
/// bubble without moving focus or the pointer (WCAG 1.4.13); leaving the control lets it
/// show again. Never put information only here: the bubble repeats or clarifies a name.
#[component]
pub fn Tooltip(
    tip: Element,
    /// Show the bubble below the control instead of above.
    #[props(default)]
    below: bool,
    /// Start dismissed, as if Escape had been pressed, e.g. when restoring a view.
    #[props(default)]
    dismissed: bool,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let id = use_id();
    use_context_provider(|| DescribedBy(id.clone()));
    let mut dismissed = use_signal(|| dismissed);
    let mut attributes = component_attributes(
        attributes,
        ClassList::new(TOOLTIP.block()).extend(class.as_deref()),
        &[],
    );
    attributes.extend(dismissed_attribute(dismissed()));
    let bubble = ClassList::new(BUBBLE.block())
        .with_modifier(
            BUBBLE,
            if below {
                BubbleSide::Below
            } else {
                BubbleSide::Above
            },
        )
        .to_string();
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_POPOVER_CSS } }
        span {
            onkeydown: move |event| {
                // Only swallow Escape when it hid something, so it still closes a dialog
                // the tooltip sits in.
                if event.key() == Key::Escape && !dismissed() {
                    dismissed.set(true);
                    event.stop_propagation();
                }
            },
            onpointerleave: move |_| dismissed.set(false),
            onfocusout: move |_| dismissed.set(false),
            ..attributes,
            {children}
            span { class: bubble, role: "tooltip", id: "{id}", {tip} }
        }
    }
}

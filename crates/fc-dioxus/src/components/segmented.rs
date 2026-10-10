//! A row of mutually exclusive or independent options drawn as one control, from
//! segmented.css.
use crate::{
    core::{
        assets,
        attrs::{availability_attributes, component_attributes},
    },
    prelude::*,
    theme::use_stylesheets,
};
const BEM: Bem = Bem::new("fc-segmented");
const HIDDEN: Bem = Bem::new("fc-u-visually-hidden");

/// Whether a segmented control picks one option or any number.
#[derive(Debug, Clone, PartialEq)]
pub enum Segments {
    /// Radios: exactly one value.
    Single {
        /// Shared input name.
        name: Id,
        /// The chosen value.
        value: Option<String>,
        /// Called with the newly chosen value.
        onchange: Option<EventHandler<String>>,
    },
    /// Checkboxes: any set of values.
    Multiple {
        /// The chosen values.
        values: Vec<String>,
        /// Called with the new set, in item order of toggling.
        onchange: Option<EventHandler<Vec<String>>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
struct SegmentContext {
    segments: Segments,
    invalid: bool,
    required: bool,
}

/// A fieldset of [`Segment`]s. The legend is visually hidden; the options say enough to
/// sighted users, but a screen reader needs the question.
#[component]
pub fn Segmented(
    legend: AccessibleName,
    segments: Segments,
    #[props(default)] invalid: bool,
    #[props(default)] required: bool,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let context = SegmentContext {
        segments,
        invalid,
        required,
    };
    let mut shared = use_context_provider(|| Signal::new(context.clone()));
    if *shared.peek() != context {
        shared.set(context);
    }
    let attributes = component_attributes(
        attributes,
        ClassList::new(BEM.block()).extend(class.as_deref()),
        &[],
    );
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_SEGMENTED_CSS } }
        fieldset { ..attributes,
            legend { class: HIDDEN.block(), "{legend}" }
            {children}
        }
    }
}

/// One option of a [`Segmented`] control.
#[component]
pub fn Segment(
    value: String,
    #[props(default)] availability: Availability,
    children: Element,
) -> Element {
    let Some(context) = try_consume_context::<Signal<SegmentContext>>().map(|c| c.read().clone())
    else {
        return rsx! {};
    };
    let blocked = availability != Availability::Enabled;
    let state = availability_attributes(&availability);
    let (kind, name, checked) = match &context.segments {
        Segments::Single {
            name,
            value: chosen,
            ..
        } => (
            "radio",
            Some(name.to_string()),
            chosen.as_deref() == Some(value.as_str()),
        ),
        Segments::Multiple { values, .. } => ("checkbox", None, values.contains(&value)),
    };
    let segments = context.segments.clone();
    let own = value.clone();
    rsx! {
        label { class: BEM.element("item"),
            input { class: BEM.element("input"), r#type: kind, name, value: value.clone(), checked,
                aria_invalid: context.invalid.then_some("true"), required: context.required,
                onclick: move |event| if blocked { event.prevent_default(); },
                onchange: move |event| match &segments {
                    Segments::Single { onchange: Some(onchange), .. } => onchange.call(own.clone()),
                    Segments::Multiple { values, onchange: Some(onchange) } => {
                        let mut next: Vec<String> = values.iter().filter(|v| **v != own).cloned().collect();
                        if event.checked() { next.push(own.clone()); }
                        onchange.call(next);
                    }
                    _ => {}
                },
                ..state,
            }
            span { class: BEM.element("check"), aria_hidden: "true" }
            {children}
        }
    }
}

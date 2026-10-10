//! Skeletons, linear progress, and the progress ring, from loading.css.
use crate::{
    core::{
        assets,
        attrs::component_attributes,
        measure::{custom_property, CustomProperty},
    },
    prelude::*,
    theme::use_stylesheets,
};
const SKELETON: Bem = Bem::new("fc-skeleton");
const PROGRESS: Bem = Bem::new("fc-progress");
const RING: Bem = Bem::new("fc-progress-ring");
fc_modifier! {
    /// The shape of the content a skeleton stands in for.
    pub enum SkeletonShape for SKELETON {
        /// One line box.
        Default => None,
        /// A line of text, slightly shorter than its line box.
        Text => "text",
        /// A block such as an image; `--fc-skeleton-size` sets its height.
        Block => "block",
        /// A circle such as an avatar.
        Circle => "circle",
    }
}
fc_modifier! { enum ProgressSize for PROGRESS { Md => None, Sm => "sm" } }
fc_modifier! {
    /// Progress ring sizes.
    pub enum RingSize for RING {
        /// Default.
        Md => None,
        /// Small, without room for a label.
        Sm => "sm",
        /// Large.
        Lg => "lg",
    }
}
fc_modifier! { enum RingTone for RING { Normal => None, Error => "error" } }

/// A placeholder in the shape of content that is still loading. It is an empty element
/// with nothing to announce; the surrounding [`Loading`] region says what is coming.
#[component]
pub fn Skeleton(
    #[props(default)] shape: SkeletonShape,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let attributes = component_attributes(
        attributes,
        ClassList::new(SKELETON.block())
            .with_modifier(SKELETON, shape)
            .extend(class.as_deref()),
        &[],
    );
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_LOADING_CSS } }
        span { ..attributes }
    }
}

/// A named busy region holding skeletons until its content arrives.
///
/// It has no class of its own: upstream styles the skeletons, and the region exists only
/// to carry `aria-busy` and say what is loading.
#[component]
pub fn Loading(
    label: AccessibleName,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let attributes: Vec<_> = attributes
        .into_iter()
        .filter(|attr| !["role", "aria-busy", "aria-label"].contains(&attr.name))
        .collect();
    rsx! {
        div { role: "group", aria_busy: "true", aria_label: label.to_string(), ..attributes, {children} }
    }
}

/// How much of a task is done, if that is known.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProgressValue {
    /// Work is happening but its extent is unknown.
    Indeterminate,
    /// `value` of `max` is done.
    Of {
        /// Completed amount.
        value: f64,
        /// Total amount.
        max: f64,
    },
}

/// Linear progress, as the native `<progress>` element assistive technology understands.
#[component]
pub fn Progress(
    label: AccessibleName,
    value: ProgressValue,
    #[props(default)] small: bool,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let mut attributes = component_attributes(
        attributes,
        ClassList::new(PROGRESS.block())
            .with_modifier(
                PROGRESS,
                if small {
                    ProgressSize::Sm
                } else {
                    ProgressSize::Md
                },
            )
            .extend(class.as_deref()),
        &["value", "max", "aria-label"],
    );
    if let ProgressValue::Of { value, max } = value {
        let max = if max > 0.0 { max } else { 1.0 };
        attributes.push(Attribute::new(
            "value",
            value.clamp(0.0, max).to_string(),
            None,
            false,
        ));
        attributes.push(Attribute::new("max", max.to_string(), None, false));
    }
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_LOADING_CSS } }
        progress { aria_label: label.to_string(), ..attributes }
    }
}

/// Circular determinate progress. An unknown amount is a [`Spinner`](super::spinner::Spinner);
/// the ring never pretends to know.
#[component]
pub fn ProgressRing(
    label: AccessibleName,
    percent: Percent,
    #[props(default)] size: RingSize,
    /// Print the percentage inside the ring. Small rings have no room for it.
    #[props(default)]
    show_label: bool,
    #[props(default)] error: bool,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let mut attributes = component_attributes(
        attributes,
        ClassList::new(RING.block())
            .with_modifier(RING, size)
            .with_modifier(
                RING,
                if error {
                    RingTone::Error
                } else {
                    RingTone::Normal
                },
            )
            .extend(class.as_deref()),
        &[
            "role",
            "aria-label",
            "aria-valuemin",
            "aria-valuemax",
            "aria-valuenow",
            "style",
        ],
    );
    // The drawn value and aria-valuenow are written together so they cannot disagree.
    attributes.push(custom_property(CustomProperty::ProgressValue, percent));
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_LOADING_CSS } }
        span { role: "progressbar", aria_label: label.to_string(), aria_valuemin: "0", aria_valuemax: "100",
            aria_valuenow: "{percent}", ..attributes,
            if show_label && size != RingSize::Sm { span { class: RING.element("label"), "{percent}%" } }
        }
    }
}

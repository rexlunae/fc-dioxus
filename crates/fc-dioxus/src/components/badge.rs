//! Status labels, interactive tags, and keyboard keys from badge.css.
use crate::{
    core::{
        assets,
        attrs::{availability_attributes, component_attributes},
    },
    prelude::*,
    theme::use_stylesheets,
};
const BEM: Bem = Bem::new("fc-badge");
const TAG: Bem = Bem::new("fc-tag");
const KBD: Bem = Bem::new("fc-kbd");
fc_modifier! {
    /// Badge tones, including upstream's checked solid accent pair.
    pub enum BadgeTone for BEM {
        /// Neutral label.
        Neutral => None,
        /// Accent tint.
        Accent => "accent",
        /// Information tint.
        Info => "info",
        /// Success tint.
        Success => "success",
        /// Warning tint.
        Warning => "warning",
        /// Error tint.
        Danger => "danger",
        /// Solid accent; does not imply a solid version of every tone.
        Solid => "solid",
    }
}
fc_modifier! { enum BadgeFill for BEM { Filled => None, Outline => "outline" } }
fc_modifier! { enum BadgeShape for BEM { Pill => None, Square => "square" } }
fc_modifier! { enum BadgeNumber for BEM { Text => None, Count => "count" } }
/// A static status label. The visible text supplies meaning independently of color.
#[component]
pub fn Badge(
    #[props(default)] tone: BadgeTone,
    #[props(default)] solid: bool,
    #[props(default)] outline: bool,
    #[props(default)] square: bool,
    #[props(default)] dot: bool,
    #[props(default)] count: bool,
    #[props(default)] clipped: bool,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let classes = ClassList::new(BEM.block())
        .with_modifier(BEM, tone)
        .with_modifier(
            BEM,
            if solid {
                BadgeTone::Solid
            } else {
                BadgeTone::Neutral
            },
        )
        .with_modifier(
            BEM,
            if outline {
                BadgeFill::Outline
            } else {
                BadgeFill::Filled
            },
        )
        .with_modifier(
            BEM,
            if square {
                BadgeShape::Square
            } else {
                BadgeShape::Pill
            },
        )
        .with_modifier(
            BEM,
            if count {
                BadgeNumber::Count
            } else {
                BadgeNumber::Text
            },
        )
        .extend(class.as_deref());
    let attributes = component_attributes(attributes, classes, &["tabindex"]);
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_BADGE_CSS } }
        span { ..attributes,
            if dot { span { class: BEM.element("dot"), aria_hidden: "true" } }
            if clipped { span { class: BEM.element("label"), {children} } } else { {children} }
        }
    }
}
/// The named removal action of a token.
#[derive(Clone, PartialEq)]
pub struct TagRemove {
    /// Name describing which token is removed.
    pub label: AccessibleName,
    /// Request removal; the owner retains state and restores focus.
    pub onremove: EventHandler<()>,
}
/// A controlled native toggle, or a token with its own named remove button.
#[component]
pub fn Tag(
    #[props(default)] pressed: bool,
    #[props(default)] onchange: Option<EventHandler<bool>>,
    #[props(default)] remove: Option<TagRemove>,
    #[props(default)] availability: Availability,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let mut attributes = component_attributes(
        attributes,
        ClassList::new(TAG.block()).extend(class.as_deref()),
        &[
            "type",
            "role",
            "aria-pressed",
            "aria-disabled",
            "disabled",
            "onclick",
        ],
    );
    let state = availability_attributes(&availability);
    let blocked = availability != Availability::Enabled;
    if remove.is_none() {
        attributes.extend(state.clone());
        attributes.push(Attribute::new(
            "aria-pressed",
            pressed.to_string(),
            None,
            false,
        ));
    }
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_BADGE_CSS } }
        if let Some(remove) = remove {
            span { ..attributes,
                span { class: TAG.element("label"), {children} }
                button { r#type: "button", class: TAG.element("remove"), aria_label: remove.label.to_string(),
                    onclick: move |event| { if blocked { event.prevent_default(); } else { remove.onremove.call(()); } },
                    ..state, "×"
                }
            }
        } else {
            button { r#type: "button",
                onclick: move |event| { if blocked { event.prevent_default(); event.stop_propagation(); } else if let Some(onchange) = onchange { onchange.call(!pressed); } },
                ..attributes, {children}
            }
        }
    }
}
/// A visible keyboard key, using native keyboard-text semantics.
#[component]
pub fn Kbd(
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let attributes = component_attributes(
        attributes,
        ClassList::new(KBD.block()).extend(class.as_deref()),
        &[],
    );
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_BADGE_CSS } }
        kbd { ..attributes, {children} }
    }
}

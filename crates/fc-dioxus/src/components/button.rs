//! Native actions styled by upstream's button.css.
use crate::{
    core::{
        assets,
        attrs::{availability_attributes, component_attributes},
    },
    prelude::*,
    theme::use_stylesheets,
};

const BEM: Bem = Bem::new("fc-btn");
fc_modifier! {
    /// All upstream button styles, including semantic aliases.
    pub enum ButtonVariant for BEM {
        /// Unaccented native action.
        Neutral => None,
        /// Main action.
        Primary => "primary",
        /// Outlined supporting action.
        Secondary => "secondary",
        /// Transparent action.
        Ghost => "ghost",
        /// Alias of ghost.
        Tertiary => "tertiary",
        /// Alias of primary.
        Ok => "ok",
        /// Alias of secondary.
        Cancel => "cancel",
        /// Tinted action.
        Subtle => "subtle",
        /// Destructive action requiring consumer confirmation.
        Danger => "danger",
    }
}
fc_modifier! {
    /// Button target size.
    pub enum ButtonSize for BEM {
        /// Standard target.
        Md => None,
        /// Compact target.
        Sm => "sm",
        /// Large target.
        Lg => "lg",
    }
}
fc_modifier! { enum ButtonWidth for BEM { Auto => None, Block => "block" } }
fc_modifier! { enum ButtonShape for BEM { Text => None, Icon => "icon" } }
/// Native form behavior; ordinary actions do not submit by default.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ButtonType {
    /// Ordinary action.
    #[default]
    Button,
    /// Submit the containing form.
    Submit,
    /// Reset the containing form.
    Reset,
}
impl ButtonType {
    fn as_str(self) -> &'static str {
        match self {
            Self::Button => "button",
            Self::Submit => "submit",
            Self::Reset => "reset",
        }
    }
}
/// Props for a native button with protected activation state.
#[derive(Props, Clone, PartialEq)]
pub struct ButtonProps {
    /// Visual style.
    #[props(default)]
    pub variant: ButtonVariant,
    /// Target size.
    #[props(default)]
    pub size: ButtonSize,
    /// Fill the container width.
    #[props(default)]
    pub block: bool,
    /// Whether activation is possible.
    #[props(default)]
    pub availability: Availability,
    /// Retain the label, show a spinner, and prevent duplicate activation.
    #[props(default)]
    pub busy: bool,
    /// Optional toggle state.
    #[props(default)]
    pub pressed: Option<bool>,
    /// Form behavior.
    #[props(default)]
    pub r#type: ButtonType,
    /// Select icon-only layout with a required accessible name.
    #[props(default)]
    pub icon_label: Option<AccessibleName>,
    /// Native activation event, guarded for unavailable and busy states.
    #[props(default)]
    pub onclick: Option<EventHandler<MouseEvent>>,
    /// Extra classes.
    #[props(default)]
    pub class: Option<String>,
    /// Consumer native attributes, before owned semantics.
    #[props(extends = GlobalAttributes, extends = button)]
    pub attributes: Vec<Attribute>,
    /// Visible label or decorative icon in icon-only mode.
    pub children: Element,
}
/// A native button that remains named while busy or unavailable.
#[component]
pub fn Button(props: ButtonProps) -> Element {
    let blocked = props.busy || props.availability != Availability::Enabled;
    let classes = ClassList::new(BEM.block())
        .with_modifier(BEM, props.variant)
        .with_modifier(BEM, props.size)
        .with_modifier(
            BEM,
            if props.block {
                ButtonWidth::Block
            } else {
                ButtonWidth::Auto
            },
        )
        .with_modifier(
            BEM,
            if props.icon_label.is_some() {
                ButtonShape::Icon
            } else {
                ButtonShape::Text
            },
        )
        .extend(super::group_item_class().as_deref())
        .extend(props.class.as_deref());
    let mut attributes = component_attributes(
        props.attributes,
        classes,
        &[
            "type",
            "role",
            "disabled",
            "aria-disabled",
            "aria-busy",
            "aria-pressed",
            "onclick",
        ],
    );
    attributes.extend(availability_attributes(&props.availability));
    // Inside a Tooltip, the bubble describes the button too.
    if let Some(super::popover::DescribedBy(tip)) = try_consume_context() {
        match attributes
            .iter_mut()
            .find(|attr| attr.name == "aria-describedby")
        {
            Some(attr) => {
                if let dioxus::core::AttributeValue::Text(ids) = &attr.value {
                    attr.value = dioxus::core::AttributeValue::Text(format!("{ids} {tip}"));
                }
            }
            None => attributes.push(Attribute::new(
                "aria-describedby",
                tip.to_string(),
                None,
                false,
            )),
        }
    }
    if let Some(label) = &props.icon_label {
        attributes.retain(|attr| !matches!(attr.name, "aria-label" | "aria-labelledby"));
        attributes.push(Attribute::new("aria-label", label.to_string(), None, false));
    }
    attributes.push(Attribute::new("type", props.r#type.as_str(), None, false));
    if props.busy {
        attributes.push(Attribute::new("aria-busy", "true", None, false));
    }
    if let Some(pressed) = props.pressed {
        attributes.push(Attribute::new(
            "aria-pressed",
            pressed.to_string(),
            None,
            false,
        ));
    }
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_BUTTON_CSS } }
        button {
            onclick: move |event: MouseEvent| {
                if blocked { event.prevent_default(); event.stop_propagation(); }
                else if let Some(handler) = props.onclick { handler.call(event); }
            },
            ..attributes,
            if props.busy { span { class: BEM.element("spinner"), aria_hidden: "true" } }
            if props.busy { span { {props.children} } }
            else { {props.children} }
        }
    }
}

/// A link drawn as a button, for actions that go somewhere: "View report", "Back to
/// projects". Inside a `dioxus-router` `Router` (feature `router`), an internal `href`
/// navigates in-app and is marked current when it is the route being shown.
///
/// It stays a link, so it opens in a new tab with a modified click and is announced as
/// a link. An unavailable link drops its `href` and carries `aria-disabled`, since `<a>`
/// has no disabled state.
#[component]
pub fn LinkButton(
    href: String,
    #[props(default)] variant: ButtonVariant,
    #[props(default)] size: ButtonSize,
    #[props(default)] block: bool,
    /// Select icon-only layout with a required accessible name.
    #[props(default)]
    icon_label: Option<AccessibleName>,
    #[props(default)] availability: Availability,
    /// Called as well as navigating, e.g. to close a menu.
    #[props(default)]
    onclick: Option<EventHandler<MouseEvent>>,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes, extends = a)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let blocked = availability != Availability::Enabled;
    let link = crate::core::link::use_link((!blocked).then_some(href));
    let classes = ClassList::new(BEM.block())
        .with_modifier(BEM, variant)
        .with_modifier(BEM, size)
        .with_modifier(
            BEM,
            if block {
                ButtonWidth::Block
            } else {
                ButtonWidth::Auto
            },
        )
        .with_modifier(
            BEM,
            if icon_label.is_some() {
                ButtonShape::Icon
            } else {
                ButtonShape::Text
            },
        )
        .extend(super::group_item_class().as_deref())
        .extend(class.as_deref());
    let mut attributes = component_attributes(
        attributes,
        classes,
        &["href", "aria-disabled", "aria-current", "onclick"],
    );
    match &availability {
        Availability::Enabled => {}
        Availability::Disabled => {
            attributes.push(Attribute::new("aria-disabled", "true", None, false));
        }
        other => attributes.extend(availability_attributes(other)),
    }
    if let Some(label) = &icon_label {
        attributes.retain(|attr| !matches!(attr.name, "aria-label" | "aria-labelledby"));
        attributes.push(Attribute::new("aria-label", label.to_string(), None, false));
    }
    let href = link.href.clone();
    let current = link.current;
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_BUTTON_CSS } }
        a { href, aria_current: current.then_some("page"),
            onclick: move |event: MouseEvent| {
                if blocked {
                    event.prevent_default();
                    return;
                }
                link.follow(&event);
                if let Some(onclick) = onclick { onclick.call(event); }
            },
            ..attributes, {children}
        }
    }
}

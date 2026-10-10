//! The floating action button and its menu, from fab.css.
use crate::{
    core::{
        assets,
        attrs::{availability_attributes, component_attributes},
    },
    prelude::*,
    theme::use_stylesheets,
};
const BEM: Bem = Bem::new("fc-fab");
const MENU: Bem = Bem::new("fc-fab-menu");
const BTN: Bem = Bem::new("fc-btn");
fc_modifier! {
    /// FAB colors.
    pub enum FabTone for BEM {
        /// The tinted default.
        Default => None,
        /// The accent fill.
        Primary => "primary",
        /// A surface color, for busy backgrounds.
        Surface => "surface",
    }
}
fc_modifier! {
    /// FAB sizes.
    pub enum FabSize for BEM {
        /// Default.
        Md => None,
        /// Small.
        Sm => "sm",
        /// Large.
        Lg => "lg",
    }
}
fc_modifier! { enum FabShape for BEM { Icon => None, Extended => "extended" } }
fc_modifier! { enum FabElevation for BEM { Raised => None, Lowered => "lowered" } }
fc_modifier! { enum FabPosition for BEM { Inline => None, Fixed => "fixed" } }

/// The screen's primary action, floating over content.
///
/// The `label` names it. Icon-only by default, where the label is its accessible name;
/// `extended` shows the label as text beside the icon instead.
#[component]
pub fn Fab(
    icon: Element,
    label: AccessibleName,
    #[props(default)] extended: bool,
    #[props(default)] tone: FabTone,
    #[props(default)] size: FabSize,
    #[props(default)] lowered: bool,
    /// Pin to the viewport's end corner.
    #[props(default)]
    fixed: bool,
    #[props(default)] availability: Availability,
    #[props(default)] busy: bool,
    /// The id of a [`FabMenu`] this button opens.
    #[props(default)]
    menu: Option<Id>,
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let blocked = busy || availability != Availability::Enabled;
    let classes = ClassList::new(BEM.block())
        .with_modifier(BEM, tone)
        .with_modifier(BEM, size)
        .with_modifier(
            BEM,
            if extended {
                FabShape::Extended
            } else {
                FabShape::Icon
            },
        )
        .with_modifier(
            BEM,
            if lowered {
                FabElevation::Lowered
            } else {
                FabElevation::Raised
            },
        )
        .with_modifier(
            BEM,
            if fixed {
                FabPosition::Fixed
            } else {
                FabPosition::Inline
            },
        )
        .extend(class.as_deref());
    let mut attributes = component_attributes(
        attributes,
        classes,
        &[
            "type",
            "aria-label",
            "aria-busy",
            "disabled",
            "aria-disabled",
        ],
    );
    attributes.extend(availability_attributes(&availability));
    if !extended {
        attributes.push(Attribute::new("aria-label", label.to_string(), None, false));
    }
    if busy {
        attributes.push(Attribute::new("aria-busy", "true", None, false));
    }
    if let Some(menu) = &menu {
        attributes.push(Attribute::new(
            "popovertarget",
            menu.to_string(),
            None,
            false,
        ));
    }
    rsx! {
        if use_stylesheets() {
            document::Stylesheet { href: assets::COMPONENTS_BUTTON_CSS }
            document::Stylesheet { href: assets::COMPONENTS_FAB_CSS }
        }
        button { r#type: "button",
            onclick: move |event: MouseEvent| {
                if blocked { event.prevent_default(); event.stop_propagation(); }
                else if let Some(onclick) = onclick { onclick.call(event); }
            },
            ..attributes,
            if busy { span { class: BTN.element("spinner"), aria_hidden: "true" } }
            else { span { class: BEM.element("icon"), aria_hidden: "true", {icon} } }
            if extended { "{label}" }
        }
    }
}

/// The actions a [`Fab`] reveals, as a popover. The FAB opens it with `menu: id`.
#[component]
pub fn FabMenu(
    id: Id,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let mut attributes = component_attributes(
        attributes,
        ClassList::new(MENU.block()).extend(class.as_deref()),
        &["id", "popover"],
    );
    attributes.push(Attribute::new("popover", "auto", None, false));
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_FAB_CSS } }
        div { id: "{id}", ..attributes, {children} }
    }
}

/// One action in a [`FabMenu`].
#[component]
pub fn FabMenuItem(
    #[props(default)] availability: Availability,
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    let onclick = availability.guard(onclick);
    let state = availability_attributes(&availability);
    rsx! {
        button { class: MENU.element("item"), r#type: "button",
            onclick: move |event: MouseEvent| if let Some(onclick) = onclick { onclick.call(event); } else { event.prevent_default(); },
            ..state, {children}
        }
    }
}

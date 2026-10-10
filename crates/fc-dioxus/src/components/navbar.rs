//! The bottom navigation bar for narrow screens, from navbar.css.
use crate::{
    core::{assets, attrs::component_attributes},
    prelude::*,
    theme::use_stylesheets,
};
const BEM: Bem = Bem::new("fc-navbar");
const HIDDEN: Bem = Bem::new("fc-u-visually-hidden");
fc_modifier! { enum NavbarPosition for BEM { Inline => None, Fixed => "fixed" } }

/// Three to five top-level destinations, each an icon over a label.
#[component]
pub fn Navbar(
    label: AccessibleName,
    /// Dock to the bottom of the viewport. The page reserves the space itself.
    #[props(default)]
    fixed: bool,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let attributes = component_attributes(
        attributes,
        ClassList::new(BEM.block())
            .with_modifier(
                BEM,
                if fixed {
                    NavbarPosition::Fixed
                } else {
                    NavbarPosition::Inline
                },
            )
            .extend(class.as_deref()),
        &["aria-label"],
    );
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_NAVBAR_CSS } }
        nav { aria_label: label.to_string(), ..attributes, {children} }
    }
}

/// A badge on a navigation icon. The badge is drawn inside the hidden icon, so what it
/// means is always also said in words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavBadge {
    /// A number, e.g. unread messages.
    Count {
        /// The number shown.
        count: u32,
        /// What it means, read after the label, e.g. "12 unread".
        announce: AccessibleName,
    },
    /// A dot for "something new".
    Dot {
        /// What it means, e.g. "new activity".
        announce: AccessibleName,
    },
}

/// One destination in a [`Navbar`].
#[component]
pub fn NavbarItem(
    icon: Element,
    label: Element,
    #[props(default)] href: Option<String>,
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
    #[props(default)] current: bool,
    #[props(default)] badge: Option<NavBadge>,
    #[props(default)] availability: Availability,
) -> Element {
    let (count, announce) = match &badge {
        Some(NavBadge::Count { count, announce }) => {
            (Some(count.to_string()), Some(announce.to_string()))
        }
        Some(NavBadge::Dot { announce }) => (Some(String::new()), Some(announce.to_string())),
        None => (None, None),
    };
    let body = rsx! {
        span { class: BEM.element("icon"), aria_hidden: "true",
            {icon}
            if let Some(count) = count { span { class: BEM.element("badge"), "{count}" } }
        }
        span { class: BEM.element("label"), {label} }
        if let Some(announce) = announce { span { class: HIDDEN.block(), ", {announce}" } }
    };
    super::nav_target(
        BEM.element("item"),
        href,
        onclick,
        current,
        &availability,
        Vec::new(),
        body,
    )
}

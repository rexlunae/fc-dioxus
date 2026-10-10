//! The application header, from masthead.css.
use crate::{
    core::{assets, attrs::component_attributes},
    prelude::*,
    theme::use_stylesheets,
};
const BEM: Bem = Bem::new("fc-masthead");

/// The top bar: brand, primary navigation, and global actions.
#[component]
pub fn Masthead(
    brand: Element,
    /// Where the brand links, usually the home page.
    #[props(default)]
    href: Option<String>,
    /// A decorative logo mark before the brand name.
    #[props(default)]
    mark: Option<Element>,
    /// Names the navigation landmark.
    nav_label: AccessibleName,
    /// [`MastheadLink`]s.
    #[props(default)]
    nav: Option<Element>,
    #[props(default)] actions: Option<Element>,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let attributes = component_attributes(
        attributes,
        ClassList::new(BEM.block()).extend(class.as_deref()),
        &[],
    );
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_MASTHEAD_CSS } }
        header { ..attributes,
            a { class: BEM.element("brand"), href,
                if let Some(mark) = mark { span { class: BEM.element("mark"), aria_hidden: "true", {mark} } }
                {brand}
            }
            if let Some(nav) = nav {
                nav { aria_label: nav_label.to_string(), ul { class: BEM.element("nav"), {nav} } }
            }
            if let Some(actions) = actions { div { class: BEM.element("actions"), {actions} } }
        }
    }
}

/// One destination in a [`Masthead`]: a link, or a button given `onclick` and no `href`.
#[component]
pub fn MastheadLink(
    #[props(default)] href: Option<String>,
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
    #[props(default)] current: bool,
    #[props(default)] availability: Availability,
    children: Element,
) -> Element {
    rsx! {
        li { {super::nav_target(BEM.element("link"), href, onclick, current, &availability, Vec::new(), children)} }
    }
}

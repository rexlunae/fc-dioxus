//! The path to the current page, from breadcrumb.css.
use crate::{
    core::{assets, attrs::component_attributes},
    prelude::*,
    theme::use_stylesheets,
};
const BEM: Bem = Bem::new("fc-breadcrumb");

/// An ordered trail of [`Crumb`]s inside a named navigation landmark.
#[component]
pub fn Breadcrumb(
    /// Names the landmark; "Breadcrumb" when omitted.
    #[props(default)]
    label: Option<AccessibleName>,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let attributes = component_attributes(
        attributes,
        ClassList::new(BEM.block()).extend(class.as_deref()),
        &["aria-label"],
    );
    let label = label.map_or_else(|| "Breadcrumb".to_owned(), |l| l.to_string());
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_BREADCRUMB_CSS } }
        nav { aria_label: label, ..attributes,
            ol { class: BEM.element("list"), {children} }
        }
    }
}

/// One step in a [`Breadcrumb`]. A crumb without `href` is the current page.
#[component]
pub fn Crumb(
    #[props(default)] href: Option<String>,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let current = href.is_none();
    let link = crate::core::link::use_link(href);
    let href = link.href.clone();
    let attributes = component_attributes(
        attributes,
        ClassList::new(&BEM.element("link")).extend(class.as_deref()),
        &["href", "aria-current"],
    );
    rsx! {
        li { class: BEM.element("item"),
            a { href, aria_current: current.then_some("page"), onclick: move |event| { link.follow(&event); }, ..attributes, {children} }
        }
    }
}

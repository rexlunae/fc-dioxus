//! The navigation sidebar and its icon rail, from nav.css. Collapsing ports upstream's
//! `nav.ts`.
use crate::{
    core::{
        assets,
        attrs::{availability_attributes, component_attributes},
    },
    prelude::*,
    theme::use_stylesheets,
};
const BEM: Bem = Bem::new("fc-nav");
const LIST: Bem = Bem::new("fc-nav__list");
fc_modifier! { enum ListDepth for LIST { Top => None, Nested => "nested" } }

#[derive(Clone, Copy)]
struct Rail(Signal<bool>);

/// A sidebar of links that can collapse to an icon rail.
///
/// The toggle's `aria-expanded` is the single state: the stylesheet reads it, so what is
/// announced and what is drawn cannot disagree. Collapsing hides labels visually but
/// keeps them as each link's name, and gives each link a `title` so a mouse user can
/// read what an icon stands for.
#[component]
pub fn NavSidebar(
    label: AccessibleName,
    title: Element,
    /// Names the collapse toggle; "Sidebar" when omitted. Without `collapsible` there is
    /// no toggle.
    #[props(default)]
    toggle_label: Option<AccessibleName>,
    #[props(default = true)] collapsible: bool,
    #[props(default)] expanded: Option<ReadSignal<bool>>,
    #[props(default = true)] default_expanded: bool,
    #[props(default)] ontoggle: Option<EventHandler<bool>>,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let state = use_controllable(expanded, || default_expanded, ontoggle);
    let id = use_id();
    let mut rail = use_context_provider(|| Rail(Signal::new(!default_expanded)));
    let open = state.get();
    if *rail.0.peek() == open {
        rail.0.set(!open);
    }
    let attributes = component_attributes(
        attributes,
        ClassList::new(BEM.block()).extend(class.as_deref()),
        &["id", "aria-label"],
    );
    let toggle_label = toggle_label.map_or_else(|| "Sidebar".to_owned(), |l| l.to_string());
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_NAV_CSS } }
        nav { id: "{id}", aria_label: label.to_string(), ..attributes,
            div { class: BEM.element("header"),
                span { class: BEM.element("title"), {title} }
                if collapsible {
                    button { class: BEM.element("toggle"), r#type: "button", aria_label: toggle_label,
                        aria_expanded: open.to_string(), aria_controls: "{id}",
                        onclick: move |_| state.set(!open),
                    }
                }
            }
            {children}
        }
    }
}

/// A small heading above a [`NavList`].
#[component]
pub fn NavHeading(children: Element) -> Element {
    rsx! { p { class: BEM.element("heading"), {children} } }
}

/// A list of [`NavLink`]s. `nested` indents it under the link it belongs to.
#[component]
pub fn NavList(#[props(default)] nested: bool, children: Element) -> Element {
    let class = ClassList::new(LIST.block())
        .with_modifier(
            LIST,
            if nested {
                ListDepth::Nested
            } else {
                ListDepth::Top
            },
        )
        .to_string();
    rsx! { ul { class, {children} } }
}

/// What a link has to do with where the user is.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NavCurrent {
    /// Elsewhere.
    #[default]
    No,
    /// The current page (`aria-current="page"`).
    Page,
    /// Contains the current page, e.g. its section (`aria-current="true"`).
    Section,
}

/// A destination. The label stays the accessible name in the rail.
#[component]
pub fn NavLink(
    label: String,
    #[props(default)] href: Option<String>,
    #[props(default)] icon: Option<Element>,
    /// A count or note after the label.
    #[props(default)]
    meta: Option<Element>,
    #[props(default)] current: NavCurrent,
    #[props(default)] availability: Availability,
    /// A nested [`NavList`] under this link.
    #[props(default)]
    nested: Option<Element>,
) -> Element {
    let rail = try_consume_context::<Rail>().is_some_and(|rail| *rail.0.read());
    let blocked = availability != Availability::Enabled;
    let state = match &availability {
        Availability::Disabled => vec![Attribute::new("aria-disabled", "true", None, false)],
        other => availability_attributes(other),
    };
    let current = match current {
        NavCurrent::No => None,
        NavCurrent::Page => Some("page"),
        NavCurrent::Section => Some("true"),
    };
    rsx! {
        li {
            a { class: BEM.element("link"), href: href.filter(|_| !blocked), aria_current: current,
                // In the rail the label is hidden visually, so the title names the icon.
                title: rail.then(|| label.clone()),
                ..state,
                {icon}
                span { class: BEM.element("label"), "{label}" }
                if let Some(meta) = meta { span { class: BEM.element("meta"), {meta} } }
            }
            {nested}
        }
    }
}

/// A 20-unit stroke icon for a [`NavLink`]. Children are SVG shapes such as `path`.
#[component]
pub fn NavIcon(children: Element) -> Element {
    rsx! {
        svg { class: BEM.element("icon"), view_box: "0 0 20 20", fill: "none", stroke: "currentColor",
            stroke_width: "1.6", "aria-hidden": "true", {children}
        }
    }
}

/// A collapsible group of links under a label.
#[component]
pub fn NavGroup(label: Element, #[props(default)] open: bool, children: Element) -> Element {
    rsx! {
        details { class: BEM.element("group"), open,
            summary { class: BEM.element("group-label"), {label} }
            NavList { {children} }
        }
    }
}

/// Links pinned to the bottom of the sidebar, such as settings.
#[component]
pub fn NavFooter(children: Element) -> Element {
    rsx! { div { class: BEM.element("footer"), NavList { {children} } } }
}

//! Rows of one to three lines with optional leading and trailing content, from list.css.
use crate::{
    core::{
        assets,
        attrs::{availability_attributes, clipped_attributes, component_attributes},
    },
    prelude::*,
    theme::use_stylesheets,
};
const BEM: Bem = Bem::new("fc-list");
fc_modifier! { enum ListRule for BEM { Plain => None, Divided => "divided" } }

/// A native list of [`ListItem`]s.
#[component]
pub fn List(
    #[props(default)] divided: bool,
    /// Render `<ol>`, when the order is itself information.
    #[props(default)]
    ordered: bool,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let attributes = component_attributes(
        attributes,
        ClassList::new(BEM.block())
            .with_modifier(
                BEM,
                if divided {
                    ListRule::Divided
                } else {
                    ListRule::Plain
                },
            )
            .extend(class.as_deref()),
        &[],
    );
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_LIST_CSS } }
        if ordered { ol { ..attributes, {children} } } else { ul { ..attributes, {children} } }
    }
}

/// What activating a list row does.
#[derive(Debug, Default, Clone, PartialEq)]
#[non_exhaustive]
pub enum ListAction {
    /// The row is static content.
    #[default]
    None,
    /// The headline is a link covering the whole row.
    Link(String),
    /// The headline is a button covering the whole row.
    Button(EventHandler<()>),
    /// The row is a `<label>` for the control in `trailing`, such as a switch.
    Label,
}

/// One row of a [`List`].
#[component]
pub fn ListItem(
    headline: Element,
    #[props(default)] overline: Option<Element>,
    #[props(default)] supporting: Option<Element>,
    /// Supporting text the CSS clamps; truncation is declared and the full text kept
    /// in `title`. Used instead of `supporting`.
    #[props(default)]
    supporting_clipped: Option<Clipped>,
    /// Decorative leading content: an avatar, an icon. Hidden from assistive technology.
    #[props(default)]
    leading: Option<Element>,
    #[props(default)] trailing: Option<Element>,
    /// Hide `trailing` from assistive technology, e.g. a chevron.
    #[props(default)]
    trailing_decorative: bool,
    #[props(default)] action: ListAction,
    /// The current page for a link, or the current item for a button.
    #[props(default)]
    current: bool,
    #[props(default)] availability: Availability,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let item = BEM.element("item");
    let headline_class = ClassList::new(&BEM.element("action"))
        .with(BEM.element("headline"))
        .to_string();
    let blocked = availability != Availability::Enabled;
    // Links have no disabled attribute, so both kinds of unavailability are ARIA there.
    let link_state = match &availability {
        Availability::Disabled => vec![Attribute::new("aria-disabled", "true", None, false)],
        state => availability_attributes(state),
    };
    let button_state = availability_attributes(&availability);
    let headline = match action.clone() {
        ListAction::Link(href) => {
            let link = crate::core::link::use_link((!blocked).then_some(href));
            let current = current || link.current;
            let href = link.href.clone();
            rsx! {
                a { class: headline_class, href, aria_current: current.then_some("page"),
                    onclick: move |event| { link.follow(&event); }, ..link_state, {headline} }
            }
        }
        ListAction::Button(onclick) => rsx! {
            button { class: headline_class, r#type: "button", aria_current: current.then_some("true"),
                onclick: move |_| if !blocked { onclick.call(()); }, ..button_state, {headline} }
        },
        ListAction::None | ListAction::Label => {
            rsx! { span { class: BEM.element("headline"), {headline} } }
        }
    };
    let supporting = match (supporting_clipped, supporting) {
        (Some(text), _) => {
            let clipped = clipped_attributes(&text);
            let shown = text.shown();
            rsx! { span { class: BEM.element("supporting"), ..clipped, "{shown}" } }
        }
        (None, Some(supporting)) => {
            rsx! { span { class: BEM.element("supporting"), {supporting} } }
        }
        (None, None) => rsx! {},
    };
    let body = rsx! {
        if let Some(leading) = leading { span { class: BEM.element("leading"), aria_hidden: "true", {leading} } }
        span { class: BEM.element("content"),
            if let Some(overline) = overline { span { class: BEM.element("overline"), {overline} } }
            {headline}
            {supporting}
        }
        if let Some(trailing) = trailing {
            span { class: BEM.element("trailing"), aria_hidden: trailing_decorative.then_some("true"), {trailing} }
        }
    };
    if action == ListAction::Label {
        let attributes = component_attributes(
            attributes,
            ClassList::new(&item).extend(class.as_deref()),
            &[],
        );
        rsx! { li { label { ..attributes, {body} } } }
    } else {
        let attributes = component_attributes(
            attributes,
            ClassList::new(&item).extend(class.as_deref()),
            &[],
        );
        rsx! { li { ..attributes, {body} } }
    }
}

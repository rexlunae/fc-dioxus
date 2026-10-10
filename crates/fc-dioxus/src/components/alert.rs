//! Static and announced feedback from alert.css.
use crate::{
    core::{assets, attrs::component_attributes},
    prelude::*,
    theme::use_stylesheets,
};
const BEM: Bem = Bem::new("fc-alert");
fc_modifier! {
    /// Feedback tones provided by upstream.
    pub enum AlertTone for BEM {
        /// Neutral information.
        Neutral => None,
        /// Informational callout.
        Info => "info",
        /// Successful result.
        Success => "success",
        /// Something needs attention.
        Warning => "warning",
        /// An error or destructive consequence.
        Danger => "danger",
    }
}
fc_modifier! { enum AlertLayout for BEM { Box => None, Banner => "banner" } }
/// Whether newly inserted feedback should be announced.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Urgency {
    /// Ordinary content, with no live-region role.
    #[default]
    Static,
    /// Announce after the current speech.
    Polite,
    /// Interrupt for an urgent result.
    Assertive,
}
/// Decorative icon choice. The text always carries the meaning.
#[derive(Debug, Default, Clone, PartialEq)]
pub enum AlertIcon {
    /// Upstream's text symbol for the tone.
    #[default]
    Default,
    /// No icon.
    None,
    /// A consumer icon, hidden from assistive technology.
    Custom(Element),
}
/// A dismiss control cannot be constructed without its accessible name.
#[derive(Clone, PartialEq)]
pub struct Dismiss {
    /// Name of the dismiss action.
    pub label: AccessibleName,
    /// Request dismissal; the owner controls visibility and focus restoration.
    pub ondismiss: EventHandler<()>,
}
/// An upstream alert; static by default to avoid unsolicited announcements.
#[component]
pub fn Alert(
    #[props(default)] tone: AlertTone,
    #[props(default)] title: Option<Element>,
    #[props(default)] icon: AlertIcon,
    #[props(default)] banner: bool,
    #[props(default)] actions: Option<Element>,
    #[props(default)] dismiss: Option<Dismiss>,
    #[props(default)] urgency: Urgency,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let classes = ClassList::new(BEM.block())
        .with_modifier(BEM, tone)
        .with_modifier(
            BEM,
            if banner {
                AlertLayout::Banner
            } else {
                AlertLayout::Box
            },
        )
        .extend(class.as_deref());
    let mut attributes = component_attributes(attributes, classes, &["role", "aria-live"]);
    match urgency {
        Urgency::Static => {}
        Urgency::Polite => attributes.push(Attribute::new("role", "status", None, false)),
        Urgency::Assertive => attributes.push(Attribute::new("role", "alert", None, false)),
    }
    let symbol = match tone {
        AlertTone::Success => "✓",
        AlertTone::Warning => "!",
        AlertTone::Danger => "✕",
        _ => "i",
    };
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_ALERT_CSS } }
        div { ..attributes,
            match icon {
                AlertIcon::None => rsx! {},
                AlertIcon::Default => rsx! { span { class: BEM.element("icon"), aria_hidden: "true", "{symbol}" } },
                AlertIcon::Custom(icon) => rsx! { span { class: BEM.element("icon"), aria_hidden: "true", {icon} } },
            }
            div { class: BEM.element("content"),
                if let Some(title) = title { p { class: BEM.element("title"), {title} } }
                p { class: BEM.element("body"), {children} }
                if let Some(actions) = actions { div { class: BEM.element("actions"), {actions} } }
            }
            if let Some(dismiss) = dismiss {
                button { r#type: "button", class: BEM.element("dismiss"), aria_label: dismiss.label.to_string(),
                    onclick: move |_| dismiss.ondismiss.call(()), "✕"
                }
            }
        }
    }
}

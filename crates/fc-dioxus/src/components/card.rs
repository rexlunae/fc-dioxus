//! Composable surfaces matching card.css and its reference fixtures.
use crate::{
    core::{assets, attrs::component_attributes},
    prelude::*,
    theme::use_stylesheets,
};
const BEM: Bem = Bem::new("fc-card");
const FOOTER: Bem = Bem::new("fc-card__footer");
fc_modifier! {
    /// Upstream card surface variants.
    pub enum CardVariant for BEM {
        /// Standard bordered surface.
        Default => None,
        /// Raised surface.
        Elevated => "elevated",
        /// Muted surface.
        Subtle => "subtle",
        /// Theme-specific feature surface.
        Feature => "feature",
    }
}
fc_modifier! { enum FooterDivider for FOOTER { None => None, Divided => "divided" } }
/// Native meaning of a card container.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CardTag {
    /// A self-contained item.
    #[default]
    Article,
    /// A generic surface.
    Div,
    /// A section, normally with a heading.
    Section,
    /// An item inside a list.
    Li,
    /// A selectable card wrapping a native input and its visible label.
    Label,
}
fn stylesheet() -> Element {
    rsx! { if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_CARD_CSS } } }
}
/// A card container; compose the parts to retain the native document structure.
#[component]
pub fn Card(
    #[props(default)] variant: CardVariant,
    #[props(default)] as_: CardTag,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let attributes = component_attributes(
        attributes,
        ClassList::new(BEM.block())
            .with_modifier(BEM, variant)
            .extend(class.as_deref()),
        &[],
    );
    rsx! { {stylesheet()} match as_ {
        CardTag::Article => rsx! { article { ..attributes, {children} } },
        CardTag::Div => rsx! { div { ..attributes, {children} } },
        CardTag::Section => rsx! { section { ..attributes, {children} } },
        CardTag::Li => rsx! { li { ..attributes, {children} } },
        CardTag::Label => rsx! { label { ..attributes, {children} } },
    } }
}
/// A row for the heading and supporting controls.
#[component]
pub fn CardHeader(
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let attributes = component_attributes(
        attributes,
        ClassList::new(&BEM.element("header")).extend(class.as_deref()),
        &[],
    );
    rsx! { {stylesheet()} div { ..attributes, {children} } }
}
/// A group for a card title and subtitle.
#[component]
pub fn CardHeading(
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let attributes = component_attributes(
        attributes,
        ClassList::new(&BEM.element("heading")).extend(class.as_deref()),
        &[],
    );
    rsx! { {stylesheet()} div { ..attributes, {children} } }
}
/// The content area of a card.
#[component]
pub fn CardBody(
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let attributes = component_attributes(
        attributes,
        ClassList::new(&BEM.element("body")).extend(class.as_deref()),
        &[],
    );
    rsx! { {stylesheet()} div { ..attributes, {children} } }
}
/// Container-responsive layout for media beside content.
#[component]
pub fn CardLayout(
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let attributes = component_attributes(
        attributes,
        ClassList::new(&BEM.element("layout")).extend(class.as_deref()),
        &[],
    );
    rsx! { {stylesheet()} div { ..attributes, {children} } }
}
/// A semantic heading, or a span when used inside a selectable label card.
#[component]
pub fn CardTitle(
    #[props(default = Some(HeadingLevel::H3))] level: Option<HeadingLevel>,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let attributes = component_attributes(
        attributes,
        ClassList::new(&BEM.element("title")).extend(class.as_deref()),
        &[],
    );
    rsx! { {stylesheet()} match level {
        Some(HeadingLevel::H1) => rsx! { h1 { ..attributes, {children} } },
        Some(HeadingLevel::H2) => rsx! { h2 { ..attributes, {children} } },
        Some(HeadingLevel::H3) => rsx! { h3 { ..attributes, {children} } },
        Some(HeadingLevel::H4) => rsx! { h4 { ..attributes, {children} } },
        Some(HeadingLevel::H5) => rsx! { h5 { ..attributes, {children} } },
        Some(HeadingLevel::H6) => rsx! { h6 { ..attributes, {children} } },
        None => rsx! { span { ..attributes, {children} } },
    } }
}
/// Secondary description, with an inline mode for selectable label cards.
#[component]
pub fn CardSubtitle(
    #[props(default)] inline: bool,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let attributes = component_attributes(
        attributes,
        ClassList::new(&BEM.element("subtitle")).extend(class.as_deref()),
        &[],
    );
    rsx! { {stylesheet()} if inline { span { ..attributes, {children} } } else { p { ..attributes, {children} } } }
}
/// A card image with required alternative text, or an upstream media placeholder.
#[component]
pub fn CardMedia(
    alt: Alt,
    #[props(default)] src: Option<String>,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let attributes = component_attributes(
        attributes,
        ClassList::new(&BEM.element("media")).extend(class.as_deref()),
        &["alt", "src", "aria-hidden", "aria-label", "role"],
    );
    let label = match &alt {
        Alt::Text(name) => name.to_string(),
        Alt::Decorative => String::new(),
    };
    rsx! { {stylesheet()}
        if let Some(src) = src { img { src, alt: label, ..attributes } }
        else if alt == Alt::Decorative { div { aria_hidden: "true", ..attributes } }
        else { div { role: "img", aria_label: label, ..attributes } }
    }
}
/// Card actions, optionally separated by an upstream rule.
#[component]
pub fn CardFooter(
    #[props(default)] divided: bool,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let classes = ClassList::new(FOOTER.block())
        .with_modifier(
            FOOTER,
            if divided {
                FooterDivider::Divided
            } else {
                FooterDivider::None
            },
        )
        .extend(class.as_deref());
    let attributes = component_attributes(attributes, classes, &[]);
    rsx! { {stylesheet()} div { ..attributes, {children} } }
}
/// A stretched native title link; footer controls retain their own hit areas.
#[component]
pub fn CardLink(
    href: String,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let attributes = component_attributes(
        attributes,
        ClassList::new(&BEM.element("link")).extend(class.as_deref()),
        &["href", "role"],
    );
    rsx! { {stylesheet()} a { href, ..attributes, {children} } }
}

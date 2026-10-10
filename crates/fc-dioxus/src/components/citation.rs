//! Inline citation markers and the source list they point to, from citation.css.
use crate::{
    core::{assets, attrs::component_attributes},
    prelude::*,
    theme::use_stylesheets,
};
const CITE: Bem = Bem::new("fc-cite");
const SOURCES: Bem = Bem::new("fc-sources");
const SOURCE: Bem = Bem::new("fc-source");

/// A numbered marker in running text, linking to its [`Source`]. Its name reads
/// "Source n: title", so the number alone is never all a screen reader hears.
#[component]
pub fn Cite(index: u32, title: AccessibleName, source: Id) -> Element {
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_CITATION_CSS } }
        a { class: CITE.block(), href: "#{source}", aria_label: "Source {index}: {title}", "{index}" }
    }
}

/// The ordered list of [`Source`]s cited above it.
#[component]
pub fn Sources(
    /// Names the list; "Sources" when omitted.
    #[props(default)]
    label: Option<AccessibleName>,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let attributes = component_attributes(
        attributes,
        ClassList::new(SOURCES.block()).extend(class.as_deref()),
        &["aria-label"],
    );
    let label = label.map_or_else(|| "Sources".to_owned(), |l| l.to_string());
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_CITATION_CSS } }
        ol { aria_label: label, ..attributes, {children} }
    }
}

/// One cited source: its number, title, and where it lives.
#[component]
pub fn Source(
    /// The id [`Cite`] markers link to.
    id: Id,
    index: u32,
    href: String,
    title: Element,
    #[props(default)] domain: Option<Element>,
    /// A decorative favicon or glyph.
    #[props(default)]
    icon: Option<Element>,
) -> Element {
    rsx! {
        li { class: SOURCE.block(), id: "{id}",
            a { class: SOURCE.element("link"), href,
                if let Some(icon) = icon { span { class: SOURCE.element("icon"), aria_hidden: "true", {icon} } }
                span { class: SOURCE.element("index"), "{index}" }
                span { class: SOURCE.element("title"), {title} }
                if let Some(domain) = domain { span { class: SOURCE.element("domain"), {domain} } }
            }
        }
    }
}

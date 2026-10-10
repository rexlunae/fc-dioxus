//! Native show/hide sections, from disclosure.css.
use crate::{
    core::{assets, attrs::component_attributes},
    prelude::*,
    theme::use_stylesheets,
};
const BEM: Bem = Bem::new("fc-disclosure");
fc_modifier! { enum DisclosureFrame for BEM { Plain => None, Boxed => "boxed" } }

/// A `<details>` section. The platform owns opening and closing, keyboard included;
/// `open` sets the initial state and `ontoggle` reports changes.
#[component]
pub fn Disclosure(
    summary: Element,
    #[props(default)] open: bool,
    #[props(default)] boxed: bool,
    #[props(default)] ontoggle: Option<EventHandler<()>>,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let attributes = component_attributes(
        attributes,
        ClassList::new(BEM.block())
            .with_modifier(
                BEM,
                if boxed {
                    DisclosureFrame::Boxed
                } else {
                    DisclosureFrame::Plain
                },
            )
            .extend(class.as_deref()),
        &["open"],
    );
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_DISCLOSURE_CSS } }
        details { open, ontoggle: move |_| if let Some(ontoggle) = ontoggle { ontoggle.call(()); }, ..attributes,
            summary { class: BEM.element("summary"), {summary} }
            div { class: BEM.element("content"), {children} }
        }
    }
}

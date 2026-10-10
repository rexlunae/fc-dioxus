//! Term and value pairs, from description-list.css.
use crate::{
    core::{assets, attrs::component_attributes},
    prelude::*,
    theme::use_stylesheets,
};
const BEM: Bem = Bem::new("fc-dl");
fc_modifier! {
    /// How terms sit relative to their values.
    pub enum DlLayout for BEM {
        /// Term and value side by side, falling back to stacked when narrow.
        Columns => None,
        /// Term above value.
        Stacked => "stacked",
    }
}
fc_modifier! { enum DlRule for BEM { Plain => None, Divided => "divided" } }

/// A native `<dl>` of [`DescriptionItem`]s.
#[component]
pub fn DescriptionList(
    #[props(default)] layout: DlLayout,
    #[props(default)] divided: bool,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let classes = ClassList::new(BEM.block())
        .with_modifier(BEM, layout)
        .with_modifier(
            BEM,
            if divided {
                DlRule::Divided
            } else {
                DlRule::Plain
            },
        )
        .extend(class.as_deref());
    let attributes = component_attributes(attributes, classes, &[]);
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_DESCRIPTION_LIST_CSS } }
        dl { ..attributes, {children} }
    }
}

/// One term and its value. `numeric` aligns the value's figures for comparison.
#[component]
pub fn DescriptionItem(
    term: Element,
    #[props(default)] numeric: bool,
    children: Element,
) -> Element {
    rsx! {
        div { class: BEM.element("item"),
            dt { {term} }
            dd { class: numeric.then(|| BEM.element("num")), {children} }
        }
    }
}

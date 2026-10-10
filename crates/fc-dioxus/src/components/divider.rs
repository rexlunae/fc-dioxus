//! A thematic break or a vertical separator, matching divider.css.

use crate::{
    core::{
        assets,
        class::{Bem, ClassList},
        state::Orientation,
    },
    fc_modifier,
    theme::use_stylesheets,
};
use dioxus::core::AttributeValue;
use dioxus::prelude::*;

const BEM: Bem = Bem::new("fc-divider");

fc_modifier! {
    /// The inset of a divider within its containing surface.
    pub enum DividerInset for BEM {
        /// Span the full available size.
        None => None,
        /// Inset from inline-start to align with list text.
        Inset => "inset",
        /// Inset from both inline edges.
        Middle => "middle",
    }
}

fc_modifier! {
    /// The CSS orientation vocabulary, separate from semantic orientation.
    enum DividerAxis for BEM {
        /// The default thematic break.
        Horizontal => None,
        /// An inline separator in a row.
        Vertical => "vertical",
    }
}

/// Props for a divider. Consumer attributes cannot replace separator semantics.
#[derive(Props, Clone, PartialEq)]
pub struct DividerProps {
    /// Horizontal thematic break or vertical separator.
    #[props(default)]
    pub orientation: Orientation,
    /// Full, start-inset, or middle-inset.
    #[props(default)]
    pub inset: DividerInset,
    /// Additional consumer classes, appended after upstream classes.
    #[props(default)]
    pub class: Option<String>,
    /// Additional global attributes, applied before component semantics.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// A horizontal `<hr>` or an unfocusable vertical `role="separator"`.
#[component]
pub fn Divider(props: DividerProps) -> Element {
    let axis = if props.orientation == Orientation::Vertical {
        DividerAxis::Vertical
    } else {
        DividerAxis::Horizontal
    };
    let mut classes = ClassList::new(BEM.block())
        .with_modifier(BEM, props.inset)
        .with_modifier(BEM, axis);
    for attribute in &props.attributes {
        if attribute.name == "class" {
            if let AttributeValue::Text(value) = &attribute.value {
                classes = classes.with(value);
            }
        }
    }
    let classes = classes.extend(props.class.as_deref()).to_string();
    let mut attributes: Vec<_> = props
        .attributes
        .into_iter()
        .filter(|attr| {
            !matches!(
                attr.name,
                "class" | "role" | "aria-orientation" | "tabindex"
            )
        })
        .collect();
    attributes.push(Attribute::new("class", classes, None, false));
    if props.orientation == Orientation::Vertical {
        attributes.push(Attribute::new("role", "separator", None, false));
        attributes.push(Attribute::new("aria-orientation", "vertical", None, false));
    }
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_DIVIDER_CSS } }
        if props.orientation == Orientation::Vertical {
            div { ..attributes }
        } else {
            hr { ..attributes }
        }
    }
}

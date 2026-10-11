//! Upstream's built-in SVG icon set, styled by icon.css, and icon-only buttons.
//!
//! The paths come from the pinned upstream build (`dist/icons.json`, generated into
//! [`IconName`]), so an icon here is the same drawing as upstream's `iconMarkup`.
use crate::{
    core::{
        assets,
        icons::{ICON_STROKE_WIDTH, ICON_VIEWBOX},
    },
    prelude::*,
    theme::use_stylesheets,
};

const BEM: Bem = Bem::new("fc-icon");
fc_modifier! {
    /// Icon size, relative to the surrounding font size.
    pub enum IconSize for BEM {
        /// The text's size, slightly enlarged (upstream's default).
        Md => None,
        /// The text's size.
        Sm => "sm",
        /// Larger than the text.
        Lg => "lg",
    }
}
fc_modifier! { enum IconDirection for BEM { Fixed => None, Mirror => "mirror" } }

/// An icon from upstream's set, drawn in the current text colour.
///
/// Decorative by default: it sits beside text or inside a named control, so it is
/// hidden from assistive technology rather than read twice. Give it a `label` only
/// when it stands alone and carries meaning (a status with no text); it is then an
/// image with that name. For an icon-only control, use [`IconButton`], which puts
/// the name on the button.
#[component]
pub fn Icon(
    name: IconName,
    #[props(default)] size: IconSize,
    /// Makes the icon a named image.
    #[props(default)]
    label: Option<AccessibleName>,
    #[props(default)] class: Option<String>,
) -> Element {
    let classes = ClassList::new(BEM.block())
        .with_modifier(BEM, size)
        .with_modifier(
            BEM,
            if name.mirrored() {
                IconDirection::Mirror
            } else {
                IconDirection::Fixed
            },
        )
        .extend(class.as_deref())
        .to_string();
    let labelled = label.is_some();
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_ICON_CSS } }
        // Presentation attributes as well as the class, so the icon draws before the
        // stylesheet loads; author CSS outranks them once it applies.
        svg { class: classes, view_box: ICON_VIEWBOX, fill: "none", stroke: "currentColor",
            stroke_width: ICON_STROKE_WIDTH, stroke_linecap: "round", stroke_linejoin: "round",
            role: labelled.then_some("img"),
            "aria-label": label.map(|label| label.to_string()),
            "aria-hidden": (!labelled).then_some("true"),
            "focusable": (!labelled).then_some("false"),
            path { d: name.path() }
        }
    }
}

/// An icon-only [`Button`]: square, with the accessible name required by type.
///
/// The `label` is the button's name, which a screen reader announces and voice
/// control users speak; the icon itself stays decorative. Pair it with a
/// [`Tooltip`](crate::prelude::Tooltip) when sighted users may not recognise the icon.
#[component]
pub fn IconButton(
    icon: IconName,
    label: AccessibleName,
    #[props(default)] variant: ButtonVariant,
    #[props(default)] size: ButtonSize,
    #[props(default)] availability: Availability,
    /// Shows the spinner in place of the icon and blocks activation.
    #[props(default)]
    busy: bool,
    #[props(default)] pressed: Option<bool>,
    #[props(default)] r#type: ButtonType,
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes, extends = button)] attributes: Vec<Attribute>,
) -> Element {
    // Built as ButtonProps so the consumer's attributes reach the button itself.
    let props = ButtonProps {
        variant,
        size,
        block: false,
        availability,
        busy,
        pressed,
        r#type,
        icon_label: Some(label),
        onclick,
        class,
        attributes,
        children: rsx! { if !busy { Icon { name: icon } } },
    };
    rsx! { Button { ..props } }
}

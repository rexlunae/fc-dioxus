//! Groups of related actions: connected button groups and general control groups, from
//! button-group.css and control-group.css.
use crate::{
    core::{assets, attrs::component_attributes},
    prelude::*,
    theme::use_stylesheets,
};
const BUTTONS: Bem = Bem::new("fc-button-group");
const CONTROLS: Bem = Bem::new("fc-control-group");

/// Connected buttons sharing edges, such as alignment or playback speed. Every
/// [`Button`] inside becomes an item; give toggles `pressed`.
#[component]
pub fn ButtonGroup(
    label: AccessibleName,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    super::provide_group_item(BUTTONS.element("item"), None);
    let attributes = component_attributes(
        attributes,
        ClassList::new(BUTTONS.block()).extend(class.as_deref()),
        &["role", "aria-label"],
    );
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_BUTTON_GROUP_CSS } }
        div { role: "group", aria_label: label.to_string(), ..attributes, {children} }
    }
}

fc_modifier! {
    /// How a control group lays out its items.
    pub enum ControlLayout for CONTROLS {
        /// Items wrap in a row with gaps.
        Wrap => None,
        /// Items touch and share borders, e.g. a search box between two buttons.
        Joined => "joined",
        /// Items stack in a column.
        Vertical => "vertical",
        /// No frame: a status beside its action.
        Plain => "plain",
    }
}

/// How a control group is named.
#[derive(Debug, Clone, PartialEq)]
pub enum GroupName {
    /// A `role="group"` named by `aria-label`.
    Label(AccessibleName),
    /// A `<fieldset>` with a visible legend, which can disable everything inside at once.
    Legend(Element),
}

/// Related controls of any kind. [`Button`]s, [`Input`]s, and [`NativeSelect`]s inside
/// become items; in a joined group, an `Input` also fills the remaining width.
#[component]
pub fn ControlGroup(
    name: GroupName,
    #[props(default)] layout: ControlLayout,
    /// Disable every control at once. Only a legend-named group (a fieldset) can.
    #[props(default)]
    disabled: bool,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let joined = layout == ControlLayout::Joined;
    super::provide_group_item(
        CONTROLS.element(if joined { "joined-item" } else { "item" }),
        joined.then(|| CONTROLS.element("control")),
    );
    let attributes = component_attributes(
        attributes,
        ClassList::new(CONTROLS.block())
            .with_modifier(CONTROLS, layout)
            .extend(class.as_deref()),
        &["role", "aria-label", "disabled"],
    );
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_CONTROL_GROUP_CSS } }
        match name {
            GroupName::Label(label) => rsx! {
                div { role: "group", aria_label: label.to_string(), ..attributes, {children} }
            },
            GroupName::Legend(legend) => rsx! {
                fieldset { disabled, ..attributes,
                    legend { class: CONTROLS.element("label"), {legend} }
                    {children}
                }
            },
        }
    }
}

/// A visible label for one control in a vertical [`ControlGroup`].
#[component]
pub fn ControlLabel(r#for: Id, children: Element) -> Element {
    // Formatted explicitly: an inline capture of a raw identifier (`{r#for}`)
    // is a hard error in the format-string parser of current nightlies, which
    // breaks the crate for consumers on newer toolchains than the pinned one.
    rsx! { label { class: CONTROLS.element("label"), r#for: format!("{}", r#for), {children} } }
}

//! A main action joined to a menu of related actions, from split-button.css.
use crate::{
    core::{
        assets,
        attrs::{availability_attributes, component_attributes},
    },
    prelude::*,
    theme::use_stylesheets,
};
const BEM: Bem = Bem::new("fc-split-btn");
const BTN: Bem = Bem::new("fc-btn");

/// The main action and a toggle that opens a menu of alternatives.
///
/// The toggle reports `aria-expanded` from `expanded`; render the menu itself in `menu`,
/// for example a popover the toggle opens through `ontoggle`. While the action is busy
/// the toggle stays focusable but inactive, so the group does not shift under the user.
#[component]
pub fn SplitButton(
    /// Names the group, e.g. "Send".
    label: AccessibleName,
    /// Names the toggle, e.g. "More send options".
    menu_label: AccessibleName,
    #[props(default)] variant: ButtonVariant,
    #[props(default)] size: ButtonSize,
    #[props(default)] availability: Availability,
    #[props(default)] busy: bool,
    #[props(default)] expanded: bool,
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
    #[props(default)] ontoggle: Option<EventHandler<()>>,
    #[props(default)] menu: Option<Element>,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let attributes = component_attributes(
        attributes,
        ClassList::new(BEM.block()).extend(class.as_deref()),
        &["role", "aria-label"],
    );
    // While the action is busy the toggle is inactive but stays focusable, so focus is
    // not lost; the busy action beside it is the explanation, as in upstream's markup.
    let mut toggle_state = availability_attributes(&availability);
    if busy && availability == Availability::Enabled {
        toggle_state.push(Attribute::new("aria-disabled", "true", None, false));
    }
    let toggle_blocked = busy || availability != Availability::Enabled;
    let toggle_class = ClassList::new(BTN.block())
        .with_modifier(BTN, variant)
        .with_modifier(BTN, size)
        .with(BTN.modifier_raw("icon"))
        .with(BEM.element("toggle"))
        .to_string();
    rsx! {
        if use_stylesheets() {
            document::Stylesheet { href: assets::COMPONENTS_BUTTON_CSS }
            document::Stylesheet { href: assets::COMPONENTS_SPLIT_BUTTON_CSS }
        }
        div { role: "group", aria_label: label.to_string(), ..attributes,
            Button { variant, size, availability: availability.clone(), busy, onclick, class: BEM.element("action"), {children} }
            button { class: toggle_class, r#type: "button", aria_haspopup: "menu", aria_expanded: expanded.to_string(),
                aria_label: menu_label.to_string(),
                onclick: move |event| if toggle_blocked { event.prevent_default(); } else if let Some(ontoggle) = ontoggle { ontoggle.call(()); },
                ..toggle_state,
                span { class: BEM.element("chevron"), aria_hidden: "true" }
            }
            {menu}
        }
    }
}

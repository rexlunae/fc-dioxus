//! A toolbar on the WAI-ARIA keyboard contract, from toolbar.css. A port of upstream's
//! `toolbar.ts`: one tab stop, arrows within, Home and End to the ends; natively disabled
//! items are skipped, while `aria-disabled` ones stay reachable so they can be discovered.
use crate::{
    behavior::roving::{self, use_member, Roster},
    core::{
        assets,
        attrs::{availability_attributes, component_attributes},
    },
    prelude::*,
    theme::use_stylesheets,
};
const BEM: Bem = Bem::new("fc-toolbar");
const ITEM: Bem = Bem::new("fc-toolbar__item");
fc_modifier! {
    /// Toolbar surfaces.
    pub enum ToolbarSurface for BEM {
        /// Inline with the content.
        Plain => None,
        /// A floating bar with its own surface, e.g. over a selection.
        Floating => "floating",
        /// Docked to an edge of its container.
        Docked => "docked",
    }
}
fc_modifier! { enum ItemShape for ITEM { Text => None, Icon => "icon" } }

#[derive(Clone, Copy)]
struct ToolbarContext {
    stop: Signal<Option<u64>>,
    orientation: Orientation,
}

/// A named group of controls that is a single tab stop.
#[component]
pub fn Toolbar(
    label: AccessibleName,
    #[props(default)] orientation: Orientation,
    #[props(default)] surface: ToolbarSurface,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    // Items join the roster; the context carries the shared focus stop.
    use_context_provider(Roster::new);
    use_context_provider(|| ToolbarContext {
        stop: Signal::new(None),
        orientation,
    });
    let attributes = component_attributes(
        attributes,
        ClassList::new(BEM.block())
            .with_modifier(BEM, surface)
            .extend(class.as_deref()),
        &["role", "aria-label", "aria-orientation"],
    );
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_TOOLBAR_CSS } }
        div { role: "toolbar", aria_label: label.to_string(),
            aria_orientation: (orientation == Orientation::Vertical).then_some("vertical"),
            ..attributes, {children}
        }
    }
}

/// A button in a [`Toolbar`]. Icon-only items take `icon_label`.
#[component]
pub fn ToolbarItem(
    #[props(default)] icon_label: Option<AccessibleName>,
    #[props(default)] pressed: Option<bool>,
    #[props(default)] availability: Availability,
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    let focusable = availability != Availability::Disabled;
    let joined = use_member("", focusable, availability == Availability::Enabled);
    let direction = use_direction();
    let context = try_consume_context::<ToolbarContext>();
    let class = ClassList::new(ITEM.block())
        .with_modifier(
            ITEM,
            if icon_label.is_some() {
                ItemShape::Icon
            } else {
                ItemShape::Text
            },
        )
        .to_string();
    let state = availability_attributes(&availability);
    let onclick = availability.guard(onclick);
    let (Some(mut context), Some((roster, key))) = (context, joined) else {
        return rsx! {};
    };
    // The stop is the item last used, or the first focusable one until then.
    let stop = match *context.stop.read() {
        Some(stop) => stop == key,
        None => roster.read().iter().find(|m| m.focusable).map(|m| m.key) == Some(key),
    };
    rsx! {
        button { r#type: "button", class, aria_label: icon_label.map(|l| l.to_string()),
            aria_pressed: pressed.map(|p| p.to_string()),
            tabindex: if stop { "0" } else { "-1" },
            onmounted: move |event| roster.mount(key, event.data()),
            onfocus: move |_| context.stop.set(Some(key)),
            onclick: move |event| if let Some(onclick) = onclick { onclick.call(event); },
            onkeydown: move |event| {
                let Some(step) = roving::intent(&event.key(), context.orientation, direction) else { return };
                let members = roster.peek();
                let Some(index) = members.iter().position(|m| m.key == key) else { return };
                let Some(next) = roving::target(members.len(), index, step, |i| members.get(i).is_some_and(|m| m.focusable)).and_then(|i| members.get(i)) else { return };
                event.prevent_default();
                context.stop.set(Some(next.key));
                roving::focus(next);
            },
            ..state, {children}
        }
    }
}

/// A visible text label inside a toolbar item, set in the UI face.
#[component]
pub fn ToolbarLabel(children: Element) -> Element {
    rsx! { span { class: BEM.element("label"), {children} } }
}

/// A separator between groups of toolbar items.
#[component]
pub fn ToolbarDivider() -> Element {
    rsx! { span { class: BEM.element("divider"), role: "separator" } }
}

//! Menus, menu buttons, and context menus, from menu.css, on upstream's
//! `fc-context-menu` model.
//!
//! Follows upstream's `menu.ts`: the menu is a popover in the top layer with light
//! dismiss; focus moves into it (roving, unlike a select) and always returns to its
//! invoker on close, Escape included; arrows wrap past separators and unavailable items,
//! Home and End jump, typed characters search within 500 ms; Tab closes without
//! trapping. Traversal is `fc_context_menu::Menu`, which upstream can run as WASM.
use crate::{
    core::{
        assets,
        attrs::{component_attributes, open_attribute},
        platform::{DomBridge, MenuPoint},
    },
    models::context_menu as model,
    prelude::*,
    theme::use_stylesheets,
};
const BEM: Bem = Bem::new("fc-menu");
const ITEM: Bem = Bem::new("fc-menu__item");
fc_modifier! { enum ItemTone for ITEM { Normal => None, Danger => "danger" } }

/// One line of a menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuEntry {
    /// An action.
    Item {
        /// Reported to `onselect`.
        id: String,
        /// What people read; typeahead matches it.
        label: String,
        /// A keyboard shortcut shown at the end, e.g. "⌘X". Decorative.
        shortcut: Option<String>,
        /// A destructive action.
        danger: bool,
        /// Unavailable: skipped by arrows, still announced.
        disabled: bool,
    },
    /// A setting toggled from the menu.
    Checkbox {
        /// Reported to `onselect`.
        id: String,
        /// What people read.
        label: String,
        /// Current state.
        checked: bool,
        /// Unavailable.
        disabled: bool,
    },
    /// An item that leads to another menu; `onselect` receives its id to open it.
    Submenu {
        /// Reported to `onselect`.
        id: String,
        /// What people read.
        label: String,
    },
    /// A heading over the items after it.
    Group(String),
    /// A rule between groups.
    Separator,
}
impl MenuEntry {
    /// A plain action.
    #[must_use]
    pub fn item(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self::Item {
            id: id.into(),
            label: label.into(),
            shortcut: None,
            danger: false,
            disabled: false,
        }
    }
    fn modeled(&self) -> Option<model::MenuItem> {
        let (id, label, kind, disabled, checked, shortcut) = match self {
            Self::Item {
                id,
                label,
                shortcut,
                disabled,
                ..
            } => (
                id,
                label,
                model::ItemKind::Normal,
                *disabled,
                false,
                shortcut.clone(),
            ),
            Self::Checkbox {
                id,
                label,
                checked,
                disabled,
            } => (
                id,
                label,
                model::ItemKind::Checkbox,
                *disabled,
                *checked,
                None,
            ),
            Self::Submenu { id, label } => {
                (id, label, model::ItemKind::Submenu, false, false, None)
            }
            Self::Group(_) | Self::Separator => return None,
        };
        Some(model::MenuItem {
            id: id.clone(),
            label: label.clone(),
            kind,
            disabled,
            checked,
            shortcut,
        })
    }
}

/// Where a menu opens.
#[derive(Debug, Clone, PartialEq)]
pub enum MenuAnchor {
    /// Under a trigger, at its start edge, flipping above when there is no room.
    Below(Id),
    /// At a viewport point, e.g. a right-click.
    At(MenuPoint),
}

#[derive(Clone, Copy)]
struct MenuState {
    model: Signal<Option<model::Menu>>,
    pending_focus: Signal<bool>,
    typing: Signal<u64>,
    expecting_open: Signal<bool>,
    /// Whether the last effect saw it open, so only a real close moves focus back.
    was_open: Signal<bool>,
    /// Set when Tab closed the menu. The browser has already moved focus to the next
    /// stop, and returning it to the trigger afterwards would send the user backwards.
    leaving: Signal<bool>,
}

/// A menu of actions, shown while `open`. Focus moves to the first item when it opens and
/// back to `return_focus` when it closes.
#[component]
pub fn Menu(
    label: AccessibleName,
    entries: Vec<MenuEntry>,
    open: bool,
    anchor: MenuAnchor,
    /// Where focus goes back to on close: the trigger, or the element right-clicked.
    return_focus: Id,
    onclose: EventHandler<()>,
    onselect: EventHandler<String>,
    /// Open with the last item focused, for a menu button opened by Arrow Up.
    #[props(default)]
    from_end: bool,
    /// The menu element's id, when a trigger refers to it. Generated when omitted.
    #[props(default)]
    menu_id: Option<Id>,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let generated = use_id();
    let id = menu_id.unwrap_or(generated);
    let built = model::Menu::new(entries.iter().filter_map(MenuEntry::modeled).collect()).ok();
    let mut state = use_hook(|| MenuState {
        model: Signal::new(None),
        pending_focus: Signal::new(false),
        typing: Signal::new(0),
        expecting_open: Signal::new(false),
        was_open: Signal::new(false),
        leaving: Signal::new(false),
    });
    let effect_id = id.clone();
    let effect_anchor = anchor.clone();
    let effect_return = return_focus.clone();
    use_effect(use_reactive!(|open, from_end| {
        let (id, anchor, back) = (
            effect_id.clone(),
            effect_anchor.clone(),
            effect_return.clone(),
        );
        let closing = !open && *state.was_open.peek();
        state.was_open.set(open);
        if open {
            state.expecting_open.set(true);
            spawn(async move {
                let _ = DomBridge.toggle_popover(&id, true).await;
                let point = match anchor {
                    MenuAnchor::At(point) => Ok(point),
                    MenuAnchor::Below(trigger) => DomBridge.point_below(&trigger).await,
                };
                if let Ok(point) = point {
                    let _ = DomBridge.place_at(&id, point).await;
                }
            });
            // A new opening is a new search, and starts at an end.
            let current = state.model.peek().clone();
            if let Some(mut menu) = current {
                if from_end {
                    menu.focus_last();
                } else {
                    menu.focus_first();
                }
                menu.reset_typeahead();
                state.model.set(Some(menu));
            }
            state.pending_focus.set(true);
        } else if closing {
            let restore = !*state.leaving.peek();
            state.leaving.set(false);
            spawn(async move {
                let _ = DomBridge.toggle_popover(&id, false).await;
                if restore {
                    let _ = DomBridge.focus(&back).await;
                }
            });
        }
    }));
    // The model follows the entries; the highlight survives when the item is still there.
    // Compared item by item, not by count: a same-length list with different or newly
    // disabled items must not keep steering arrows and typeahead by the old one.
    if state.model.peek().as_ref().map(|m| signature(m.items()))
        != built.as_ref().map(|m| signature(m.items()))
        || state.model.peek().is_none()
    {
        let active = state
            .model
            .peek()
            .as_ref()
            .and_then(model::Menu::active_index);
        let mut next = built.clone();
        if let Some(menu) = &mut next {
            menu.adopt_state(active, String::new());
        }
        state.model.set(next);
    }
    let active = state
        .model
        .read()
        .as_ref()
        .and_then(model::Menu::active_index);
    let base = id.to_string();
    let item_id = move |index: usize| format!("{base}-item-{index}");
    let focus_id = id.clone();
    use_effect(move || {
        if (state.pending_focus)() {
            state.pending_focus.set(false);
            let target = state
                .model
                .peek()
                .as_ref()
                .and_then(model::Menu::active_index);
            let menu_id = focus_id.clone();
            spawn(async move {
                let target = match target {
                    Some(index) => Id::try_from(format!("{menu_id}-item-{index}")).ok(),
                    // A menu with nothing actionable still takes focus, rather than leaving it
                    // on the trigger behind an open menu.
                    None => Some(menu_id),
                };
                if let Some(target) = target {
                    let _ = DomBridge.focus(&target).await;
                }
            });
        }
    });
    let mut attributes = component_attributes(
        attributes,
        ClassList::new(BEM.block()).extend(class.as_deref()),
        &["role", "aria-label", "popover", "tabindex"],
    );
    attributes.push(Attribute::new("popover", "auto", None, false));
    attributes.extend(open_attribute(open.then_some(true)));
    let mut modeled = 0_usize;
    let rows: Vec<(MenuEntry, Option<usize>)> = entries
        .iter()
        .map(|entry| {
            let index = entry.modeled().map(|_| {
                modeled += 1;
                modeled - 1
            });
            (entry.clone(), index)
        })
        .collect();
    let mut step = move |change: &dyn Fn(&mut model::Menu)| {
        if let Some(menu) = state.model.write().as_mut() {
            menu.reset_typeahead();
            change(menu);
        }
        state.pending_focus.set(true);
    };
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_MENU_CSS } }
        div { id: "{id}", role: "menu", aria_label: label.to_string(), tabindex: "-1",
            ontoggle: move |_| {
                // The platform's light dismiss and Escape close the popover; opening fires
                // the same event, so the first one after an open is skipped.
                if (state.expecting_open)() { state.expecting_open.set(false); } else if open { onclose.call(()); }
            },
            onkeydown: move |event| {
                let key = event.key();
                match key {
                    Key::ArrowDown => { event.prevent_default(); step(&|m| m.move_by(1)); }
                    Key::ArrowUp => { event.prevent_default(); step(&|m| m.move_by(-1)); }
                    Key::Home => { event.prevent_default(); step(&|m| m.focus_first()); }
                    Key::End => { event.prevent_default(); step(&|m| m.focus_last()); }
                    Key::Escape => { event.prevent_default(); onclose.call(()); }
                    Key::Tab => { state.leaving.set(true); onclose.call(()); }
                    Key::Character(text) if !event.modifiers().ctrl() && !event.modifiers().meta() && !event.modifiers().alt() && text != " " => {
                        event.prevent_default();
                        let moved = state.model.write().as_mut().is_some_and(|m| m.typeahead(&text));
                        if moved { state.pending_focus.set(true); }
                        // Characters typed within 500 ms of each other form one search.
                        let generation = (state.typing)() + 1;
                        state.typing.set(generation);
                        spawn(async move {
                            if DomBridge.delay(500).await.is_ok() && (state.typing)() == generation {
                                if let Some(menu) = state.model.write().as_mut() { menu.reset_typeahead(); }
                            }
                        });
                    }
                    _ => {}
                }
            },
            ..attributes,
            for (row, (entry, index)) in rows.into_iter().enumerate() {
                {entry_view(entry, index.map(|i| (i, item_id(i), active == Some(i))), row, onselect, onclose)}
            }
        }
    }
}

fn entry_view(
    entry: MenuEntry,
    item: Option<(usize, String, bool)>,
    row: usize,
    onselect: EventHandler<String>,
    onclose: EventHandler<()>,
) -> Element {
    let check = BEM.element("check");
    let label_class = BEM.element("label");
    match (entry, item) {
        (MenuEntry::Group(label), _) => {
            rsx! { div { key: "{row}", class: BEM.element("group-label"), role: "presentation", "{label}" } }
        }
        (MenuEntry::Separator, _) => {
            rsx! { hr { key: "{row}", class: BEM.element("separator"), role: "separator" } }
        }
        (
            MenuEntry::Item {
                id,
                label,
                shortcut,
                danger,
                disabled,
            },
            Some((_, dom_id, active)),
        ) => {
            let class = ClassList::new(ITEM.block())
                .with_modifier(
                    ITEM,
                    if danger {
                        ItemTone::Danger
                    } else {
                        ItemTone::Normal
                    },
                )
                .to_string();
            rsx! {
                button { key: "{row}", r#type: "button", class, role: "menuitem", id: dom_id, tabindex: if active { "0" } else { "-1" },
                    aria_disabled: disabled.then_some("true"),
                    onclick: move |_| if !disabled { onselect.call(id.clone()); onclose.call(()); },
                    span { class: check, aria_hidden: "true" }
                    span { class: label_class, "{label}" }
                    if let Some(shortcut) = shortcut { span { class: BEM.element("shortcut"), aria_hidden: "true", "{shortcut}" } }
                }
            }
        }
        (
            MenuEntry::Checkbox {
                id,
                label,
                checked,
                disabled,
            },
            Some((_, dom_id, active)),
        ) => rsx! {
            button { key: "{row}", r#type: "button", class: ITEM.block(), role: "menuitemcheckbox", id: dom_id,
                tabindex: if active { "0" } else { "-1" }, aria_checked: checked.to_string(), aria_disabled: disabled.then_some("true"),
                onclick: move |_| if !disabled { onselect.call(id.clone()); onclose.call(()); },
                span { class: check, aria_hidden: "true", if checked { "✓" } }
                span { class: label_class, "{label}" }
            }
        },
        (MenuEntry::Submenu { id, label }, Some((_, dom_id, active))) => rsx! {
            button { key: "{row}", r#type: "button", class: ITEM.block(), role: "menuitem", id: dom_id,
                tabindex: if active { "0" } else { "-1" }, aria_haspopup: "menu",
                onclick: move |_| onselect.call(id.clone()),
                span { class: check, aria_hidden: "true" }
                span { class: label_class, "{label}" }
                span { class: BEM.element("submenu-arrow"), aria_hidden: "true" }
            }
        },
        _ => rsx! {},
    }
}

/// A button that opens a menu beneath itself, on the WAI-ARIA menu button pattern:
/// click, Enter, or Space toggle it; Arrow Down opens at the first item and Arrow Up at
/// the last. `aria-expanded` follows the menu.
#[component]
pub fn MenuButton(
    /// Names the menu.
    label: AccessibleName,
    entries: Vec<MenuEntry>,
    onselect: EventHandler<String>,
    #[props(default)] variant: ButtonVariant,
    #[props(default)] size: ButtonSize,
    /// Select icon-only layout with a required accessible name.
    #[props(default)]
    icon_label: Option<AccessibleName>,
    #[props(default)] availability: Availability,
    children: Element,
) -> Element {
    let trigger = use_id();
    let mut open = use_signal(|| false);
    let mut from_end = use_signal(|| false);
    let blocked = availability != Availability::Enabled;
    let menu_id = trigger.suffixed("menu");
    rsx! {
        // The arrow keys reach the trigger's wrapper by bubbling; Button keeps its own
        // click and availability handling.
        span {
            onkeydown: move |event: KeyboardEvent| {
                if blocked { return; }
                match event.key() {
                    Key::ArrowDown => { event.prevent_default(); from_end.set(false); open.set(true); }
                    Key::ArrowUp => { event.prevent_default(); from_end.set(true); open.set(true); }
                    _ => {}
                }
            },
            Button { variant, size, icon_label, availability, id: "{trigger}",
                aria_haspopup: "menu", aria_expanded: open().to_string(), aria_controls: "{menu_id}",
                onclick: move |_| { from_end.set(false); open.toggle(); },
                {children}
            }
        }
        Menu { label, entries, open: open(), from_end: from_end(), anchor: MenuAnchor::Below(trigger.clone()), return_focus: trigger.clone(),
            onclose: move |()| open.set(false), onselect, menu_id }
    }
}

/// A region whose right-click (or the context-menu key) opens a menu at the pointer,
/// instead of the browser's own.
#[component]
pub fn ContextMenuArea(
    label: AccessibleName,
    entries: Vec<MenuEntry>,
    onselect: EventHandler<String>,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let area = use_id();
    let mut at = use_signal(|| None::<MenuPoint>);
    let attributes = component_attributes(
        attributes,
        ClassList::default().extend(class.as_deref()),
        &["id"],
    );
    let attributes: Vec<_> = attributes
        .into_iter()
        .filter(|a| {
            !(a.name == "class"
                && matches!(&a.value, dioxus::core::AttributeValue::Text(t) if t.is_empty()))
        })
        .collect();
    rsx! {
        div { id: "{area}", tabindex: "0",
            oncontextmenu: move |event| {
                event.prevent_default();
                let point = event.client_coordinates();
                at.set(Some(MenuPoint { x: point.x, y: point.y, flip_y: None, edge: None }));
            },
            ..attributes,
            {children}
        }
        Menu { label, entries, open: at().is_some(),
            anchor: MenuAnchor::At(at().unwrap_or(MenuPoint { x: 0.0, y: 0.0, flip_y: None, edge: None })),
            return_focus: area.clone(), onclose: move |()| at.set(None), onselect }
    }
}

/// What navigation depends on in a menu item; `MenuItem` has no `PartialEq` upstream.
fn signature(items: &[model::MenuItem]) -> Vec<(&str, &str, bool, bool, bool)> {
    items
        .iter()
        .map(|item| {
            (
                item.id.as_str(),
                item.label.as_str(),
                item.disabled,
                item.checked,
                matches!(item.kind, model::ItemKind::Separator),
            )
        })
        .collect()
}

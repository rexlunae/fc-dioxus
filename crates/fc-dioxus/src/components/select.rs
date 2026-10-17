//! A value-selecting combobox and its listbox, from select.css, on upstream's
//! `fc-select` model.
//!
//! Focus never leaves the invoker: the highlighted option is conveyed with
//! `aria-activedescendant`, so the control stays one tab stop and reads as a combobox.
//! Escape restores the value in effect before the list opened; Enter, Space, a click,
//! or Tab commits. Navigation, typeahead, and the committed value come from
//! `fc_select::Select`, the same model upstream's binding can run as WASM.
use crate::{
    core::{
        assets,
        attrs::{active_attribute, component_attributes, placeholder_attribute, value_attribute},
        platform::DomBridge,
    },
    models::select as model,
    prelude::*,
    theme::use_stylesheets,
};
const BEM: Bem = Bem::new("fc-select");

/// One choice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectChoice {
    /// Stable value, carried back on change. Not the label.
    pub value: String,
    /// What people read.
    pub label: String,
    /// The group heading it sits under.
    pub group: Option<String>,
    /// A second line.
    pub description: Option<String>,
    /// A short supplementary value, such as a price.
    pub meta: Option<String>,
    /// A decorative glyph before the label.
    pub icon: Option<String>,
    /// Unavailable: skipped by arrows and typeahead, never chosen, still announced.
    pub disabled: bool,
}
impl SelectChoice {
    /// A plain choice.
    #[must_use]
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            group: None,
            description: None,
            meta: None,
            icon: None,
            disabled: false,
        }
    }
    /// Under a group heading.
    #[must_use]
    pub fn in_group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }
    /// With a second line.
    #[must_use]
    pub fn describe(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
    /// With a short supplementary value, such as a price.
    #[must_use]
    pub fn meta(mut self, meta: impl Into<String>) -> Self {
        self.meta = Some(meta.into());
        self
    }
    /// With a decorative glyph.
    #[must_use]
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
    /// Unavailable.
    #[must_use]
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

fn build(choices: &[SelectChoice], committed: Option<&str>) -> model::Select {
    let options = choices
        .iter()
        .map(|c| {
            let option = model::Option_::new(&c.value, &c.label);
            let option = match &c.group {
                Some(group) => option.in_group(group),
                None => option,
            };
            if c.disabled {
                option.disabled()
            } else {
                option
            }
        })
        .collect();
    let mut select = model::Select::new(options);
    // Seed the committed value; opening and closing without a highlight keeps it.
    select.open(committed);
    select.close(false);
    select
}

/// A combobox that opens a listbox of [`SelectChoice`]s.
#[component]
pub fn Select(
    label: AccessibleName,
    options: Vec<SelectChoice>,
    #[props(default)] value: Option<ReadSignal<Option<String>>>,
    #[props(default)] default_value: Option<String>,
    #[props(default)] onchange: Option<EventHandler<String>>,
    /// Shown while nothing is chosen.
    #[props(default = "Select…".to_owned())]
    placeholder: String,
    #[props(default)] availability: Availability,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let state = use_controllable(
        value,
        || default_value,
        onchange.map(|onchange| {
            EventHandler::new(move |v: Option<String>| {
                if let Some(v) = v {
                    onchange.call(v);
                }
            })
        }),
    );
    let committed = state.get();
    let mut select = use_signal(|| build(&options, committed.as_deref()));
    // A new option list rebuilds the model; the committed value carries over.
    let mut seen = use_signal(|| options.clone());
    if *seen.peek() != options {
        seen.set(options.clone());
        select.set(build(&options, committed.as_deref()));
    }
    let mut open = use_signal(|| false);
    let mut typing = use_signal(|| 0_u64);
    let invoker = use_id();
    let list = invoker.suffixed("listbox");
    let blocked = availability != Availability::Enabled || options.is_empty();
    let model = select.read().clone();
    let active = model.active();
    let current = committed
        .as_deref()
        .and_then(|v| options.iter().find(|o| o.value == v));
    let option_id = |index: usize| format!("{list}-opt-{index}");

    // Held in signals so the handlers below are `Copy` and can be shared.
    let ids = use_signal(|| (list.clone(), invoker.clone()));
    let show = move |visible: bool, restore_focus: bool| {
        let (list, invoker) = ids.peek().clone();
        spawn(async move {
            if visible {
                let _ = DomBridge.toggle_popover(&list, true).await;
                let _ = DomBridge.place_below(&list, &invoker).await;
            } else {
                let _ = DomBridge.toggle_popover(&list, false).await;
                if restore_focus {
                    let _ = DomBridge.focus(&invoker).await;
                }
            }
        });
    };
    let mut open_list = move || {
        if open() {
            return;
        }
        select.write().open(state.get().as_deref());
        open.set(true);
        show(true, false);
    };
    // A blur or Tab already has a destination. Restoring focus there steals
    // it from the next control and can leave another selector open but unfocused.
    let mut close_list = move |commit: bool, restore_focus: bool| {
        if !open() {
            return;
        }
        select.write().close(commit);
        open.set(false);
        let after = select.peek().committed().map(ToOwned::to_owned);
        if commit && after != state.get() {
            state.set(after);
        }
        show(false, restore_focus);
    };
    let mut attributes = component_attributes(
        attributes,
        ClassList::new(BEM.block()).extend(class.as_deref()),
        &[
            "role",
            "aria-expanded",
            "aria-controls",
            "aria-activedescendant",
            "aria-label",
            "aria-disabled",
        ],
    );
    if let Some(value) = &committed {
        attributes.push(value_attribute(value));
    }
    let mut groups_seen: Option<String> = None;
    let mut rows = Vec::new();
    for &index in model.visible() {
        let Some(choice) = options.get(index) else {
            continue;
        };
        if choice.group.is_some() && choice.group != groups_seen {
            groups_seen.clone_from(&choice.group);
            rows.push((None, Some(choice.group.clone().unwrap_or_default())));
        }
        rows.push((Some(index), None));
    }
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_SELECT_CSS } }
        button { r#type: "button", id: "{invoker}", role: "combobox", aria_label: label.to_string(),
            aria_expanded: open().to_string(),
            aria_controls: open().then(|| list.to_string()),
            aria_activedescendant: if open() { active.map(option_id) } else { None },
            aria_disabled: blocked.then_some("true"),
            onclick: move |_| if !blocked { if open() { close_list(true, true) } else { open_list() } },
            onfocusout: move |_| close_list(false, false),
            onkeydown: move |event| {
                if blocked { return; }
                let key = event.key();
                match key {
                    Key::ArrowDown | Key::ArrowUp => {
                        event.prevent_default();
                        if open() {
                            select.write().move_by(if key == Key::ArrowDown { 1 } else { -1 });
                        } else {
                            open_list();
                        }
                    }
                    Key::Home | Key::End if open() => {
                        event.prevent_default();
                        if key == Key::Home { select.write().focus_first(); } else { select.write().focus_last(); }
                    }
                    Key::Enter => {
                        event.prevent_default();
                        if open() { close_list(true, true) } else { open_list() }
                    }
                    Key::Escape if open() => {
                        event.prevent_default();
                        close_list(false, true);
                    }
                    Key::Tab => close_list(true, false),
                    Key::Character(text) if open() && !event.modifiers().ctrl() && !event.modifiers().meta() && !event.modifiers().alt() => {
                        event.prevent_default();
                        if text == " " && select.peek().active().is_some() {
                            close_list(true, true);
                            return;
                        }
                        select.write().typeahead(&text);
                        // A 600 ms window lets "son" find Sonnet instead of three searches.
                        let generation = typing() + 1;
                        typing.set(generation);
                        spawn(async move {
                            if DomBridge.delay(600).await.is_ok() && typing() == generation {
                                select.write().reset_typeahead();
                            }
                        });
                    }
                    Key::Character(text) if !open() && text == " " => {
                        event.prevent_default();
                        open_list();
                    }
                    _ => {}
                }
            },
            ..attributes,
            span { class: BEM.element("value"), ..placeholder_attribute(current.is_none()),
                "{current.map_or(placeholder.as_str(), |c| c.label.as_str())}"
            }
            span { class: BEM.element("chevron"), aria_hidden: "true", "▾" }
        }
        div { class: BEM.element("list"), id: "{list}", role: "listbox", aria_label: label.to_string(), popover: "manual",
            for (option, heading) in rows {
                if let Some(heading) = heading {
                    div { class: BEM.element("group-label"), role: "presentation", "{heading}" }
                }
                if let Some(index) = option {
                    if let Some(choice) = options.get(index) {
                        {option_view(choice, index, option_id(index), committed.as_deref() == Some(choice.value.as_str()), open() && active == Some(index), select, close_list)}
                    }
                }
            }
            if model.visible().is_empty() {
                div { class: BEM.element("empty"), role: "option", aria_disabled: "true", "No matches" }
            }
        }
    }
}

fn option_view(
    choice: &SelectChoice,
    index: usize,
    id: String,
    selected: bool,
    active: bool,
    mut select: Signal<model::Select>,
    mut close: impl FnMut(bool, bool) + 'static,
) -> Element {
    let disabled = choice.disabled;
    let mut data = crate::spread![value_attribute(&choice.value)];
    data.extend(active_attribute(active));
    rsx! {
        div { class: BEM.element("option"), role: "option", id, aria_selected: selected.then_some("true"),
            aria_disabled: disabled.then_some("true"),
            // Keep focus on the invoker while the pointer is on the list.
            onmousedown: move |event| event.prevent_default(),
            onclick: move |_| if !disabled {
                let delta = {
                    let model = select.peek();
                    let from = model.active().and_then(|a| model.visible().iter().position(|&i| i == a));
                    let to = model.visible().iter().position(|&i| i == index);
                    match (from, to) {
                        (Some(from), Some(to)) => isize::try_from(to).unwrap_or(0) - isize::try_from(from).unwrap_or(0),
                        _ => 0,
                    }
                };
                select.write().move_by(delta);
                close(true, true);
            },
            ..data,
            span { class: BEM.element("option-check"), aria_hidden: "true", if selected { "✓" } }
            if let Some(icon) = &choice.icon { span { class: BEM.element("option-icon"), aria_hidden: "true", "{icon}" } }
            if let Some(description) = &choice.description {
                span { class: BEM.element("option-content"),
                    span { "{choice.label}" }
                    span { class: BEM.element("option-description"), "{description}" }
                }
            } else {
                span { "{choice.label}" }
            }
            if let Some(meta) = &choice.meta { span { class: BEM.element("option-meta"), "{meta}" } }
        }
    }
}

//! A disclosure of checkboxes with filtering and bulk actions, from multiselect.css. A
//! port of upstream's `multiselect.ts`.
use crate::{
    core::{
        assets,
        attrs::{
            availability_attributes, component_attributes, count_attribute, multiselect_part,
            selected_attribute, FcData, Kind, MultiselectPart,
        },
    },
    prelude::*,
    theme::use_stylesheets,
};
const BEM: Bem = Bem::new("fc-multiselect");
const FIELD: Bem = Bem::new("fc-field");
const LABEL: Bem = Bem::new("fc-field__label");
const INPUT: Bem = Bem::new("fc-input");
const CHECK: Bem = Bem::new("fc-check__input");
const BTN: Bem = Bem::new("fc-btn");

/// One choice.
#[derive(Debug, Clone, PartialEq)]
pub struct MultiOption {
    /// The submitted value; unique within the picker.
    pub value: String,
    /// What people read.
    pub title: String,
    /// A second line, e.g. a role.
    pub description: Option<String>,
    /// Unavailable choices keep their state: bulk actions skip them.
    pub availability: Availability,
}

/// Which options a filter query keeps, by case-insensitive substring of their text, as
/// upstream's `filter` does.
///
/// ```
/// use fc_dioxus::components::multiselect::matches;
/// assert!(matches("Ada Lovelace Maintainer", "  ada "));
/// assert!(!matches("Grace Hopper", "ada"));
/// ```
#[must_use]
pub fn matches(text: &str, query: &str) -> bool {
    text.to_lowercase().contains(&query.trim().to_lowercase())
}

/// The summary upstream shows by default: "None selected" or "n selected".
#[must_use]
pub fn default_summary(count: usize) -> String {
    if count == 0 {
        "None selected".to_owned()
    } else {
        format!("{count} selected")
    }
}

/// Native checkboxes in a `<details>`, so choosing and form submission work without
/// script; filtering, bulk actions, and the summary are added on top. Escape closes the
/// panel and returns focus to its summary.
#[component]
pub fn MultiSelect(
    /// What is being chosen, shown in the summary, e.g. "Project members".
    label: Element,
    /// The fieldset's legend.
    legend: Element,
    /// Shared `name` for form submission.
    name: String,
    options: Vec<MultiOption>,
    #[props(default)] values: Option<ReadSignal<Vec<String>>>,
    #[props(default)] default_values: Vec<String>,
    #[props(default)] onchange: Option<EventHandler<Vec<String>>>,
    /// Label for the filter field; no filter when omitted.
    #[props(default)]
    filter_label: Option<String>,
    #[props(default)] filter_placeholder: Option<String>,
    #[props(default)] open: bool,
    /// Summary text for a count; [`default_summary`] when omitted.
    #[props(default)]
    summary: Option<Callback<usize, String>>,
    /// Mirror the selection count into the `data-fc-*` contract under this id.
    #[props(default)]
    fc_id: Option<Id>,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let state = use_controllable(values, || default_values, onchange);
    let mut query = use_signal(String::new);
    // The details element is controlled: the summary's activation toggles this signal
    // instead of the native state, so Escape can close it too. Toggle events are not
    // used, because the browser also fires one for a details that starts open.
    let mut expanded = use_signal(|| open);
    // `open` stays the parent's to change after mount; the user's own toggles live in
    // `expanded` until the parent next says otherwise.
    let mut seen_open = use_signal(|| open);
    if *seen_open.peek() != open {
        seen_open.set(open);
        expanded.set(open);
    }
    let mut summary_node = use_signal(|| None::<std::rc::Rc<MountedData>>);
    let chosen = state.get();
    let mut attributes = component_attributes(
        attributes,
        ClassList::new(BEM.block()).extend(class.as_deref()),
        &["open"],
    );
    if let Some(fc_id) = fc_id {
        attributes.extend(
            FcData {
                kind: Some(Kind::Multiselect),
                id: Some(fc_id),
                state: Some(Lifecycle::Complete),
                ..FcData::default()
            }
            .attributes(),
        );
        attributes.push(count_attribute(chosen.len()));
    }
    let text = summary.map_or_else(|| default_summary(chosen.len()), |s| s.call(chosen.len()));
    let visible: Vec<bool> = options
        .iter()
        .map(|o| {
            matches(
                &format!("{} {}", o.title, o.description.clone().unwrap_or_default()),
                &query(),
            )
        })
        .collect();
    let none_visible = !visible.iter().any(|v| *v);
    let toggle = move |value: String, on: bool| {
        let mut next: Vec<String> = state.get().into_iter().filter(|v| *v != value).collect();
        if on {
            next.push(value);
        }
        state.set(next);
    };
    let editable: Vec<(String, bool)> = options
        .iter()
        .zip(&visible)
        .filter(|(o, _)| o.availability == Availability::Enabled)
        .map(|(o, v)| (o.value.clone(), *v))
        .collect();
    let select_all = {
        let editable = editable.clone();
        move |_| {
            let mut next = state.get();
            for (value, visible) in &editable {
                if *visible && !next.contains(value) {
                    next.push(value.clone());
                }
            }
            state.set(next);
        }
    };
    let clear = move |_| {
        let keep: Vec<String> = state
            .get()
            .into_iter()
            .filter(|v| !editable.iter().any(|(e, _)| e == v))
            .collect();
        state.set(keep);
    };
    rsx! {
        if use_stylesheets() {
            document::Stylesheet { href: assets::COMPONENTS_MULTISELECT_CSS }
            document::Stylesheet { href: assets::COMPONENTS_CHECK_CSS }
            document::Stylesheet { href: assets::COMPONENTS_FIELD_CSS }
            document::Stylesheet { href: assets::COMPONENTS_BUTTON_CSS }
        }
        details { open: expanded(),
            onkeydown: move |event| {
                if event.key() == Key::Escape && expanded() {
                    event.prevent_default();
                    expanded.set(false);
                    if let Some(summary) = summary_node() {
                        spawn(async move { let _ = summary.set_focus(true).await; });
                    }
                }
            },
            ..attributes,
            summary { class: BEM.element("summary"), onmounted: move |event| summary_node.set(Some(event.data())),
                onclick: move |event| { event.prevent_default(); expanded.toggle(); },
                {label} " · " span { ..crate::spread![multiselect_part(MultiselectPart::Summary)], "{text}" }
            }
            div { class: BEM.element("panel"),
                if let Some(filter_label) = filter_label {
                    label { class: FIELD.block(),
                        span { class: LABEL.block(), "{filter_label}" }
                        input { class: INPUT.block(), r#type: "search", placeholder: filter_placeholder,
                            value: query(), oninput: move |event| query.set(event.value()),
                            ..crate::spread![multiselect_part(MultiselectPart::Filter)] }
                    }
                }
                div { class: BEM.element("actions"),
                    button { class: ClassList::new(BTN.block()).with(BTN.modifier_raw("secondary")).with(BTN.modifier_raw("sm")).to_string(),
                        r#type: "button", onclick: select_all, ..crate::spread![multiselect_part(MultiselectPart::SelectAll)], "Select visible" }
                    button { class: ClassList::new(BTN.block()).with(BTN.modifier_raw("tertiary")).with(BTN.modifier_raw("sm")).to_string(),
                        r#type: "button", onclick: clear, ..crate::spread![multiselect_part(MultiselectPart::Clear)], "Clear" }
                }
                fieldset { class: BEM.element("options"),
                    legend { {legend} }
                    for (option, shown) in options.iter().zip(visible.iter().copied()) {
                        {option_view(option, shown, chosen.contains(&option.value), &name, toggle)}
                    }
                }
                p { hidden: !none_visible, ..crate::spread![multiselect_part(MultiselectPart::Empty)], "No matches. Existing selections are kept." }
            }
        }
    }
}

fn option_view(
    option: &MultiOption,
    shown: bool,
    checked: bool,
    name: &str,
    mut toggle: impl FnMut(String, bool) + 'static,
) -> Element {
    let blocked = option.availability != Availability::Enabled;
    let state = availability_attributes(&option.availability);
    let value = option.value.clone();
    rsx! {
        label { key: "{option.value}", class: BEM.element("option"), hidden: !shown, ..crate::spread![selected_attribute(checked)],
            input { class: CHECK.block(), r#type: "checkbox", name: "{name}", value: "{option.value}", checked,
                onclick: move |event| if blocked { event.prevent_default(); },
                onchange: move |event| if !blocked { toggle(value.clone(), event.checked()); },
                ..state,
            }
            span { class: BEM.element("content"),
                strong { "{option.title}" }
                if let Some(description) = &option.description { span { class: BEM.element("description"), "{description}" } }
            }
        }
    }
}

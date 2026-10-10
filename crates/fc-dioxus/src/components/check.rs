//! Checkboxes, radios, switches, and their groups, from check.css.
use crate::{
    core::{
        assets,
        attrs::{availability_attributes, component_attributes},
        platform::DomBridge,
    },
    prelude::*,
    theme::use_stylesheets,
};
const CHECK: Bem = Bem::new("fc-check");
const GROUP: Bem = Bem::new("fc-check-group");
const SWITCH: Bem = Bem::new("fc-switch");
const ROW: Bem = Bem::new("fc-switch-row");
fc_modifier! { enum GroupFlow for GROUP { Stacked => None, Inline => "inline" } }
fc_modifier! {
    /// Switch sizes.
    pub enum SwitchSize for SWITCH {
        /// Default.
        Md => None,
        /// Small.
        Sm => "sm",
        /// Large.
        Lg => "lg",
    }
}

/// A checkbox's value, including the mixed state of a parent over partly checked children.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CheckState {
    /// Unchecked.
    #[default]
    Off,
    /// Checked.
    On,
    /// Some but not all of what it governs is checked.
    Mixed,
}

fn stylesheet() -> Element {
    rsx! { if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_CHECK_CSS } } }
}

/// A labelled native checkbox.
///
/// Pass `checked` to control it, or `default_checked` to let it hold its own state.
/// `onchange` receives the new checked value; a mixed checkbox becomes checked when
/// clicked, as the platform does.
#[component]
pub fn Checkbox(
    label: Element,
    #[props(default)] hint: Option<Element>,
    #[props(default)] checked: Option<ReadSignal<CheckState>>,
    #[props(default)] default_checked: CheckState,
    #[props(default)] onchange: Option<EventHandler<bool>>,
    #[props(default)] invalid: bool,
    #[props(default)] availability: Availability,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let state = use_controllable(
        checked,
        || default_checked,
        Some(EventHandler::new(move |next: CheckState| {
            if let Some(onchange) = onchange {
                onchange.call(next == CheckState::On);
            }
        })),
    );
    let id = use_id();
    let current = state.get();
    let mixed = current == CheckState::Mixed;
    let effect_id = id.clone();
    use_effect(use_reactive!(|mixed| {
        let id = effect_id.clone();
        spawn(async move {
            // HTML forbids aria-checked on a native checkbox; the indeterminate property is
            // the only way to say "mixed", so hosts without a DOM render it unchecked.
            let _ = DomBridge.set_indeterminate(&id, mixed).await;
        });
    }));
    let blocked = availability != Availability::Enabled;
    let attributes = component_attributes(
        attributes,
        ClassList::new(CHECK.block()).extend(class.as_deref()),
        &[],
    );
    let state_attrs = availability_attributes(&availability);
    rsx! {
        {stylesheet()}
        label { ..attributes,
            input { id: "{id}", class: CHECK.element("input"), r#type: "checkbox",
                checked: current == CheckState::On,
                aria_invalid: invalid.then_some("true"),
                onclick: move |event| if blocked { event.prevent_default(); },
                onchange: move |event| if !blocked {
                    state.set(if event.checked() { CheckState::On } else { CheckState::Off });
                },
                ..state_attrs,
            }
            span { class: CHECK.element("label"), {label} }
            if let Some(hint) = hint { span { class: CHECK.element("hint"), {hint} } }
        }
    }
}

/// The radios in a [`CheckGroup`] share one name and one selected value.
#[derive(Debug, Clone, PartialEq)]
pub struct RadioChoice {
    /// The shared input name; use [`use_id`] for a unique one.
    pub name: Id,
    /// The selected value, if any.
    pub value: Option<String>,
    /// Called with the newly selected value.
    pub onchange: Option<EventHandler<String>>,
}

/// A fieldset with a legend, holding checkboxes or, with `radios`, a radio group.
#[component]
pub fn CheckGroup(
    legend: Element,
    #[props(default)] inline: bool,
    #[props(default)] radios: Option<RadioChoice>,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let mut context = use_context_provider(|| Signal::new(radios.clone()));
    if *context.peek() != radios {
        context.set(radios);
    }
    let attributes = component_attributes(
        attributes,
        ClassList::new(GROUP.block())
            .with_modifier(
                GROUP,
                if inline {
                    GroupFlow::Inline
                } else {
                    GroupFlow::Stacked
                },
            )
            .extend(class.as_deref()),
        &[],
    );
    rsx! {
        {stylesheet()}
        fieldset { ..attributes,
            legend { class: GROUP.element("legend"), {legend} }
            {children}
        }
    }
}

/// Shares radio state with the [`Radio`]s and [`Choice`]s inside, without drawing a
/// group, for layouts that bring their own fieldset.
///
/// [`Choice`]: super::field::Choice
#[component]
pub fn RadioScope(radios: RadioChoice, children: Element) -> Element {
    let mut context = use_context_provider(|| Signal::new(Some(radios.clone())));
    if context.peek().as_ref() != Some(&radios) {
        context.set(Some(radios));
    }
    rsx! { {children} }
}

/// One choice in a [`CheckGroup`] given `radios`.
#[component]
pub fn Radio(
    label: Element,
    value: String,
    #[props(default)] hint: Option<Element>,
    #[props(default)] availability: Availability,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let choice = try_consume_context::<Signal<Option<RadioChoice>>>()
        .and_then(|context| context.read().clone());
    let name = choice.as_ref().map(|c| c.name.to_string());
    let checked = choice
        .as_ref()
        .is_some_and(|c| c.value.as_deref() == Some(value.as_str()));
    let onchange = choice.and_then(|c| availability.guard(c.onchange));
    let blocked = availability != Availability::Enabled;
    let attributes = component_attributes(
        attributes,
        ClassList::new(CHECK.block()).extend(class.as_deref()),
        &[],
    );
    let state_attrs = availability_attributes(&availability);
    rsx! {
        {stylesheet()}
        label { ..attributes,
            input { class: CHECK.element("input"), r#type: "radio", name, value: value.clone(), checked,
                onclick: move |event| if blocked { event.prevent_default(); },
                onchange: move |_| if let Some(onchange) = onchange { onchange.call(value.clone()); },
                ..state_attrs,
            }
            span { class: CHECK.element("label"), {label} }
            if let Some(hint) = hint { span { class: CHECK.element("hint"), {hint} } }
        }
    }
}

/// How a standalone control gets its accessible name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlName {
    /// Named directly, with `aria-label`.
    Named(AccessibleName),
    /// Named by the `<label>` that wraps it, such as a [`ListAction::Label`] row.
    ///
    /// [`ListAction::Label`]: super::list::ListAction::Label
    Wrapped,
}
impl From<AccessibleName> for ControlName {
    fn from(name: AccessibleName) -> Self {
        Self::Named(name)
    }
}

#[component]
fn SwitchInput(
    label: ControlName,
    size: SwitchSize,
    checked: bool,
    onchange: EventHandler<bool>,
    invalid: bool,
    availability: Availability,
    class: Option<String>,
    attributes: Vec<Attribute>,
) -> Element {
    let blocked = availability != Availability::Enabled;
    let mut attributes = component_attributes(
        attributes,
        ClassList::new(SWITCH.block())
            .with_modifier(SWITCH, size)
            .extend(class.as_deref()),
        &["type", "role", "checked", "aria-label"],
    );
    attributes.extend(availability_attributes(&availability));
    rsx! {
        input { r#type: "checkbox", role: "switch", checked,
            aria_label: match label { ControlName::Named(name) => Some(name.to_string()), ControlName::Wrapped => None },
            aria_invalid: invalid.then_some("true"),
            onclick: move |event| if blocked { event.prevent_default(); },
            onchange: move |event| if !blocked { onchange.call(event.checked()); },
            ..attributes,
        }
    }
}

/// An on/off control that takes effect immediately, as `<input type=checkbox role=switch>`.
#[component]
pub fn Switch(
    #[props(into)] label: ControlName,
    #[props(default)] size: SwitchSize,
    #[props(default)] checked: Option<ReadSignal<bool>>,
    #[props(default)] default_checked: bool,
    #[props(default)] onchange: Option<EventHandler<bool>>,
    #[props(default)] invalid: bool,
    #[props(default)] availability: Availability,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let state = use_controllable(checked, || default_checked, onchange);
    rsx! {
        {stylesheet()}
        SwitchInput { label, size, checked: state.get(),
            onchange: move |value| state.set(value), invalid, availability, class, attributes }
    }
}

/// A settings row: a label and hint on one side, a switch on the other, all one target.
#[component]
pub fn SwitchRow(
    label: Element,
    #[props(default)] hint: Option<Element>,
    #[props(default)] size: SwitchSize,
    #[props(default)] checked: Option<ReadSignal<bool>>,
    #[props(default)] default_checked: bool,
    #[props(default)] onchange: Option<EventHandler<bool>>,
    #[props(default)] availability: Availability,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let state = use_controllable(checked, || default_checked, onchange);
    let attributes = component_attributes(
        attributes,
        ClassList::new(ROW.block()).extend(class.as_deref()),
        &[],
    );
    rsx! {
        {stylesheet()}
        label { ..attributes,
            span { class: ROW.element("text"),
                span { class: ROW.element("label"), {label} }
                if let Some(hint) = hint { span { class: ROW.element("hint"), {hint} } }
            }
            SwitchInput { label: ControlName::Wrapped, size, checked: state.get(), onchange: move |value| state.set(value),
                invalid: false, availability, class: None, attributes: Vec::new() }
        }
    }
}

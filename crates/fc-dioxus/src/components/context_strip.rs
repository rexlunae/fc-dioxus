//! Provider, model, and effort as one piece of state, from context-strip.css. The rules
//! are [`crate::behavior::context`], ported from upstream's `context-strip.ts`.
use crate::{
    behavior::context::{ContextModel, ContextState, Effort, ProviderChoice},
    core::{
        assets,
        attrs::{applicable_attribute, component_attributes, context_attributes},
    },
    prelude::*,
    theme::use_stylesheets,
};
const BEM: Bem = Bem::new("fc-context-strip");

/// The controls above a composer: provider, model, effort, and the model's parameters.
///
/// Changing provider resolves the model (restoring the one this provider had last time).
/// Effort applies only to reasoning models; for others the control is dimmed and
/// labelled rather than removed, and a value the user set is kept, not cleared.
#[component]
pub fn ContextStrip(
    providers: Vec<ProviderChoice>,
    #[props(default)] onchange: Option<EventHandler<ContextState>>,
    /// Hide the provider control when there is only one provider.
    #[props(default = true)]
    show_provider: bool,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let mut strip = use_signal(|| ContextModel::new(providers.clone()));
    if strip.peek().providers() != providers.as_slice() {
        strip.set(ContextModel::new(providers.clone()));
    }
    let model = strip.read().clone();
    let state = model.state();
    let reasoning = model.model().is_some_and(|m| m.reasoning);
    let mut attributes = component_attributes(
        attributes,
        ClassList::new(BEM.block()).extend(class.as_deref()),
        &[],
    );
    attributes.extend(context_attributes(
        &state.provider,
        &state.model,
        state.effort.map(Effort::as_str),
    ));
    let mut update = move |change: &dyn Fn(&mut ContextModel)| {
        let before = strip.peek().state();
        change(&mut strip.write());
        let after = strip.peek().state();
        if after != before {
            if let Some(onchange) = onchange {
                onchange.call(after);
            }
        }
    };
    let provider_options: Vec<SelectChoice> = model
        .providers()
        .iter()
        .map(|p| SelectChoice::new(p.id.clone(), p.name.clone()))
        .collect();
    let model_options: Vec<SelectChoice> = model
        .provider()
        .map(|p| {
            p.models
                .iter()
                .map(|m| SelectChoice::new(m.id.clone(), m.name.clone()))
                .collect()
        })
        .unwrap_or_default();
    let effort_options: Vec<SelectChoice> = Effort::ALL
        .iter()
        .map(|e| SelectChoice::new(e.as_str(), e.label()))
        .collect();
    // The selects are controlled by the strip, which alone resolves their values.
    let provider_value = use_memo(move || Some(strip.read().state().provider));
    let model_value = use_memo(move || Some(strip.read().state().model));
    let effort_value =
        use_memo(move || strip.read().stored_effort().map(|e| e.as_str().to_owned()));
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_CONTEXT_STRIP_CSS } }
        div { ..attributes,
            if show_provider {
                div { class: BEM.element("field"),
                    span { class: BEM.element("label"), "Provider" }
                    Select { label: crate::name!("Provider"), options: provider_options,
                        value: ReadSignal::from(provider_value),
                        onchange: move |v: String| update(&|m| m.set_provider(&v)) }
                }
            }
            div { class: BEM.element("field"),
                span { class: BEM.element("label"), "Model" }
                Select { label: crate::name!("Model"), options: model_options, value: ReadSignal::from(model_value),
                    onchange: move |v: String| update(&|m| m.set_model(&v)) }
            }
            // Dimmed, never removed: a row that reflows when the model changes makes the
            // strip jump, and a missing control is not announced at all.
            div { class: BEM.element("field"), ..crate::spread![applicable_attribute(reasoning)],
                span { class: BEM.element("label"), "Effort" }
                Select { label: crate::name!("Effort"), options: effort_options, placeholder: "n/a",
                    value: ReadSignal::from(effort_value),
                    availability: if reasoning { Availability::Enabled } else { Availability::Disabled },
                    onchange: move |v: String| if let Some(e) = Effort::from_value(&v) { update(&|m| m.set_effort(e)) } }
            }
            span { class: BEM.element("spacer") }
            div { class: BEM.element("params"),
                for (name, value) in state.parameters.iter() {
                    span { key: "{name}", class: BEM.element("param"),
                        span { class: BEM.element("param-name"), "{name}" }
                        span { class: BEM.element("param-value"), "{value}" }
                    }
                }
            }
            span { class: BEM.element("notice"), role: "status", hidden: model.notice().is_none(),
                "{model.notice().unwrap_or_default()}"
            }
        }
    }
}

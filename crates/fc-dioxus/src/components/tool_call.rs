//! A recorded or running operation, from tool-call.css.
use crate::{
    core::{
        assets,
        attrs::{clipped_attributes, component_attributes, FcData, Kind},
        measure::{custom_property, CustomProperty},
    },
    prelude::*,
    theme::use_stylesheets,
};
const BEM: Bem = Bem::new("fc-tool");

/// How far a running tool call has got.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToolProgress {
    /// Working, extent unknown.
    Indeterminate,
    /// This much is done.
    Done(Percent),
}

/// One tool invocation: its name, state, and duration, then progress, its payload, or why
/// it failed. Every field is real text and the state is `data-fc-state`, so the record
/// reads the same to a person and to automation, and survives greyscale.
#[component]
pub fn ToolCall(
    name: String,
    state: Lifecycle,
    /// Elapsed time, e.g. "412 ms".
    #[props(default)]
    duration: Option<String>,
    #[props(default)] progress: Option<ToolProgress>,
    /// A small label above the payload, e.g. "Arguments".
    #[props(default)]
    payload_label: Option<Element>,
    /// Arguments or result. Truncation is declared, and the full value kept in `title`.
    #[props(default)]
    payload: Option<Clipped>,
    #[props(default)] error: Option<Element>,
    #[props(default)] record: Record,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let mut attributes = component_attributes(
        attributes,
        ClassList::new(BEM.block()).extend(class.as_deref()),
        &[],
    );
    attributes.extend(
        FcData {
            kind: Some(Kind::ToolCall),
            id: record.id.clone(),
            seq: record.seq,
            state: Some(state),
            ts: record.ts.clone(),
            ..FcData::default()
        }
        .attributes(),
    );
    let state_attr = FcData {
        state: Some(state),
        ..FcData::default()
    }
    .attributes();
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_TOOL_CALL_CSS } }
        div { ..attributes,
            div { class: BEM.element("header"),
                span { class: BEM.element("name"), "{name}" }
                span { class: BEM.element("state"), ..state_attr, "{state.as_str()}" }
                if let Some(duration) = duration { span { class: BEM.element("duration"), "{duration}" } }
            }
            if let Some(progress) = progress {
                div { class: BEM.element("progress"),
                    div { class: BEM.element("progress-track"),
                        match progress {
                            ToolProgress::Indeterminate => rsx! {
                                div { class: BEM.element("progress-bar"), role: "progressbar", aria_label: "{name} progress" }
                            },
                            ToolProgress::Done(percent) => {
                                let width = custom_property(CustomProperty::ToolProgress, format!("{percent}%"));
                                rsx! {
                                    div { class: BEM.element("progress-bar"), role: "progressbar", aria_label: "{name} progress",
                                        aria_valuemin: "0", aria_valuemax: "100", aria_valuenow: "{percent}", ..vec![width] }
                                }
                            }
                        }
                    }
                }
            }
            if let Some(label) = payload_label { span { class: BEM.element("label"), {label} } }
            if let Some(payload) = payload {
                div { class: BEM.element("payload"), ..clipped_attributes(&payload), "{payload.shown()}" }
            }
            if let Some(error) = error { div { class: BEM.element("error"), {error} } }
        }
    }
}

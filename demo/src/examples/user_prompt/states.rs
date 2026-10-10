use super::super::record;
use fc_dioxus::prelude::*;

// example:start
/// An agent's question in every lifecycle state. Once settled, the answer is locked.
#[component]
pub fn States() -> Element {
    const GRID: Bem = Bem::new("fc-grid-auto");
    const LABEL: Bem = Bem::new("fc-field__label");
    let description = use_id();
    rsx! {
        div { class: GRID.block(),
            for (index, state) in Lifecycle::ALL.iter().copied().enumerate() {
                Question { key: "{index}", state, index }
            }
        }
        div { class: GRID.block(),
            UserPrompt { label: name!("Organize the workspace"), region: true, record: record("user-prompt", Lifecycle::Pending, 100),
                title: rsx! { "Choose how to organize the extended multilingual workspace session 日本語 العربية" },
                label { class: LABEL.block(), r#for: "{description}", "Project description" }
                Input { id: "{description}", name: "description", default_value: "An extended project description with localized text 日本語 العربية" }
            }
            UserPrompt { label: name!("No questions"), region: true, record: record("user-prompt", Lifecycle::Complete, 101),
                title: rsx! { "No questions" }, p { "The agent has not requested an answer." } }
        }
        LivePrompt {}
    }
}

/// One prompt, holding its own answer.
#[component]
fn Question(state: Lifecycle, index: usize) -> Element {
    let name = use_id();
    let mut workspace = use_signal(|| "current".to_owned());
    let locked = if state.is_settled() {
        Availability::Disabled
    } else {
        Availability::Enabled
    };
    rsx! {
        UserPrompt { label: name!("Choose a workspace"), record: record("user-prompt", state, index),
            status: rsx! { p { "{state:?}" } }, title: rsx! { "Choose a workspace" },
            actions: rsx! {
                Button { variant: ButtonVariant::Primary, r#type: ButtonType::Submit, availability: locked.clone(), "Answer" }
                Button { variant: ButtonVariant::Secondary, r#type: ButtonType::Reset, availability: locked.clone(), "Reset" }
            },
            fieldset {
                legend { "Where should the agent work?" }
                RadioScope { radios: RadioChoice { name, value: Some(workspace()), onchange: Some(EventHandler::new(move |v| workspace.set(v))) },
                    Radio { label: rsx! { "Current project" }, value: "current", availability: locked.clone() }
                    Radio { label: rsx! { "New directory" }, value: "new", availability: locked.clone() }
                }
            }
        }
    }
}

/// A prompt you can answer: submitting settles it.
#[component]
fn LivePrompt() -> Element {
    let mut answered = use_signal(|| false);
    let mut depth = use_signal(String::new);
    let state = if answered() {
        Lifecycle::Complete
    } else {
        Lifecycle::Pending
    };
    rsx! {
        UserPrompt { label: name!("How thorough?"), title: rsx! { "How thorough should the review be?" },
            record: Record { state: Some(state), ..Record::default() },
            meta: rsx! { if answered() { "Answered: {depth}" } else { "Asked by the review agent." } },
            onsubmit: move |()| answered.set(true),
            actions: rsx! { Button { variant: ButtonVariant::Primary, r#type: ButtonType::Submit, availability: if answered() { Availability::Disabled } else { Availability::Enabled }, "Answer" } },
            Field { label: rsx! { "Depth" }, hint: rsx! { "For example: skim, normal, line by line." },
                Input { value: ReadSignal::from(depth), oninput: move |v| depth.set(v), readonly: answered() }
            }
        }
    }
}

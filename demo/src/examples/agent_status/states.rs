use fc_dioxus::prelude::*;

// example:start
/// The status line in every lifecycle state, plus a live one you can advance.
#[component]
pub fn States() -> Element {
    const STACK: Bem = Bem::new("fc-stack");
    let mut step = use_signal(|| 0_usize);
    let live = Lifecycle::ALL
        .get(step())
        .copied()
        .unwrap_or(Lifecycle::Complete);
    rsx! {
        div { class: STACK.block(),
            for state in [Lifecycle::Pending, Lifecycle::Queued, Lifecycle::Running, Lifecycle::Streaming,
                Lifecycle::Complete, Lifecycle::Aborted, Lifecycle::Denied, Lifecycle::Error]
            {
                AgentStatus { state, label: rsx! { "{state:?} step" }, meta: rsx! { "4s" } }
            }
            Button { variant: ButtonVariant::Secondary, onclick: move |_| step.set((step() + 1) % Lifecycle::ALL.len()), "Advance the live status" }
            AgentStatus { state: live, label: rsx! { "Indexing the repository ({live:?})" } }
        }
    }
}

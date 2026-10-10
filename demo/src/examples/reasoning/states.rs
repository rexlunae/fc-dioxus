use fc_dioxus::prelude::*;

// example:start
/// A reasoning trace while pending and streaming (open), and after it settles.
#[component]
pub fn States() -> Element {
    const STACK: Bem = Bem::new("fc-stack");
    rsx! {
        div { class: STACK.block(),
            for (state, open) in [(Lifecycle::Pending, false), (Lifecycle::Streaming, true), (Lifecycle::Complete, false),
                (Lifecycle::Aborted, false), (Lifecycle::Error, false)]
            {
                Reasoning { state, open, duration: rsx! { "12s" }, "Cite both sources, and note the figure is uncompressed." }
            }
        }
    }
}

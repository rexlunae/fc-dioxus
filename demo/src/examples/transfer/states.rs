use super::super::{long_path, record};
use fc_dioxus::prelude::*;

// example:start
/// A download in every lifecycle state: waiting, streaming, running with progress, and
/// each way it can end.
#[component]
pub fn States() -> Element {
    const GRID: Bem = Bem::new("fc-grid-auto");
    rsx! {
        div { class: GRID.block(),
            for (index, state) in Lifecycle::ALL.iter().copied().enumerate() {
                Transfer { key: "{index}", label: name!("Model download"), record: record("transfer", state, index),
                    status: rsx! { p { "{state:?}" } }, title: rsx! { "Model download" },
                    meta: rsx! { bdi { "models/example.gguf" } " · {progress_text(state)}" },
                    progress: match state {
                        Lifecycle::Pending | Lifecycle::Queued => Some(ProgressValue::Indeterminate),
                        Lifecycle::Running => Some(ProgressValue::Of { value: 40.0, max: 100.0 }),
                        _ => None,
                    },
                    actions: matches!(state, Lifecycle::Pending | Lifecycle::Queued | Lifecycle::Running).then(|| rsx! {
                        Button { variant: ButtonVariant::Secondary, availability: Availability::Disabled, "Cancel" }
                    }),
                }
            }
            Transfer { label: name!("Download with unknown total"), region: true, record: record("transfer", Lifecycle::Pending, 100),
                title: rsx! { "Download with a long destination" }, progress: ProgressValue::Indeterminate,
                p { bdi { "{long_path()}" } }
            }
            Transfer { label: name!("No transfers"), region: true, record: record("transfer", Lifecycle::Complete, 101),
                title: rsx! { "No transfers" }, p { "No downloads have been started." } }
        }
    }
}

fn progress_text(state: Lifecycle) -> &'static str {
    match state {
        Lifecycle::Pending | Lifecycle::Queued => "Waiting to start · total unknown",
        Lifecycle::Streaming | Lifecycle::Running => "40 MB of 100 MB",
        Lifecycle::Complete => "100 MB received · complete",
        Lifecycle::Aborted => "Cancelled after 40 MB",
        Lifecycle::Denied => "Transfer not authorized",
        _ => "Failed after 40 MB · connection interrupted",
    }
}

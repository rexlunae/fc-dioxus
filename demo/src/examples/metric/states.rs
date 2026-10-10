use super::super::record;
use fc_dioxus::prelude::*;

// example:start
/// A metric in every lifecycle state. The value is machine-readable through `<data>`, and
/// the record is mirrored into `data-fc-*` for automation.
#[component]
pub fn States() -> Element {
    const GRID: Bem = Bem::new("fc-grid-auto");
    let mut tokens = use_signal(|| 0_u32);
    rsx! {
        div { class: GRID.block(),
            for (index, state) in Lifecycle::ALL.iter().copied().enumerate() {
                Metric { key: "{index}", label: name!("Input tokens"), record: record("metric", state, index),
                    status: rsx! { p { "{state:?}" } },
                    title: rsx! { "Input tokens" }, value: Some(12480.0), display: "12,480",
                    meta: rsx! { "This session · exact total" } }
            }
            Metric { label: name!("Requests"), region: true, record: record("metric", Lifecycle::Pending, 100),
                title: rsx! { "Requests for the extended multilingual workspace session 日本語 العربية" },
                value: Some(12_345_678_901_234_567_890.0), display: "12,345,678,901,234,567,890" }
            Metric { label: name!("Session requests"), region: true, record: record("metric", Lifecycle::Complete, 101),
                title: rsx! { "Session requests" }, value: None, display: "Unavailable" }
            Metric { label: name!("Messages sent"), title: rsx! { "Messages sent" },
                value: Some(f64::from(tokens())), display: tokens().to_string(), meta: rsx! { "Since you opened this page" },
                actions: rsx! {
                    Button { variant: ButtonVariant::Secondary, size: ButtonSize::Sm, onclick: move |_| tokens += 1, "Send one" }
                    Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, onclick: move |_| tokens.set(0), "Reset" }
                } }
        }
    }
}

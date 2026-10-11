use fc_dioxus::prelude::*;

// example:start
/// Every author role with message actions, then the lifecycle states a message passes
/// through, including a truncated one that keeps its full text.
#[component]
pub fn States() -> Element {
    const STACK: Bem = Bem::new("fc-stack");
    let mut rating = use_signal(|| Some(true));
    let reason = use_id();
    let sent = || {
        Timestamp::iso8601("2026-10-06T09:12:00Z")
            .ok()
            .map(|at| SentAt {
                at,
                display: "09:12".into(),
            })
    };
    rsx! {
        div { class: STACK.block(),
            Chat { label: name!("Roles"),
                ChatGroup {
                    Message { role: MessageRole::User, state: Lifecycle::Complete, author: rsx! { "You" }, sent: sent(),
                        p { "Which services are degraded?" } }
                    Message { role: MessageRole::Assistant, state: Lifecycle::Complete, author: rsx! { "Assistant" }, sent: sent(),
                        persistent_actions: true,
                        actions: rsx! {
                            MessageAction { icon_label: name!("Copy"), Icon { name: IconName::Copy } }
                            MessageAction { icon_label: name!("Good response"), pressed: rating() == Some(true), onclick: move |_| rating.set(Some(true)), "👍" }
                            MessageAction { icon_label: name!("Bad response"), pressed: rating() == Some(false), onclick: move |_| rating.set(Some(false)), "👎" }
                            MessageAction { availability: Availability::Unavailable { reason: reason.clone() }, "↻ Retry" }
                        },
                        p { "One: " strong { "api" } " in " code { "us-east-1" } "." } }
                    Message { role: MessageRole::System, state: Lifecycle::Complete, author: rsx! { "System" }, sent: sent(),
                        p { "Model switched to Fast." } }
                    Message { role: MessageRole::Tool, state: Lifecycle::Complete, author: rsx! { "search_logs" }, sent: sent(),
                        p { "14 matching lines." } }
                }
            }
            p { id: "{reason}", "Retry is available once the current run finishes." }
            Chat { label: name!("States"),
                ChatGroup {
                    Message { role: MessageRole::Assistant, state: Lifecycle::Pending, author: rsx! { "Assistant" }, sent: sent(), p { "Waiting for the model…" } }
                    Message { role: MessageRole::Assistant, state: Lifecycle::Streaming, author: rsx! { "Assistant" }, sent: sent(), p { "Checking the worker queue…" } }
                    Message { role: MessageRole::Assistant, state: Lifecycle::Aborted, author: rsx! { "Assistant" }, sent: sent(), p { "Stopped by you." } }
                    Message { role: MessageRole::System, state: Lifecycle::Error, author: rsx! { "System" }, sent: sent(), p { "Metrics fetch failed; retrying in 30s." } }
                    Message { role: MessageRole::Assistant, state: Lifecycle::Complete, author: rsx! { "Assistant" }, sent: sent(),
                        clipped: Clipped::chars("Full response: 14 log lines, 3 tool calls, 1 error.", 36),
                        p { "Full response: 14 log lines, 3 tool…" } }
                }
            }
            Chat { label: name!("Message parts"),
                ChatGroup {
                    Message { role: MessageRole::Assistant, state: Lifecycle::Running, author: rsx! { "Assistant" },
                        MessagePart { p { "Looking that up." } }
                        MessagePart { ToolCall { name: "search_logs", state: Lifecycle::Running, progress: ToolProgress::Indeterminate } }
                    }
                }
            }
        }
    }
}

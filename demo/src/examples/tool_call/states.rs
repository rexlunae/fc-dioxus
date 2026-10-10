use fc_dioxus::prelude::*;

// example:start
/// Running with known and unknown progress, complete with a full and a truncated payload,
/// and each way a call can fail.
#[component]
pub fn States() -> Element {
    const STACK: Bem = Bem::new("fc-stack");
    let ms = || Some("412\u{a0}ms".to_owned());
    rsx! {
        div { class: STACK.block(),
            ToolCall { name: "query_metrics", state: Lifecycle::Running, duration: ms(), progress: ToolProgress::Done(Percent::new(62.0)) }
            ToolCall { name: "index_repo", state: Lifecycle::Running, duration: ms(), progress: ToolProgress::Indeterminate }
            ToolCall { name: "search_logs", state: Lifecycle::Complete, duration: ms(),
                payload: Clipped::full(r#"{ "service": "api", "window": "15m" }"#) }
            ToolCall { name: "read_file", state: Lifecycle::Complete, duration: ms(),
                payload: Clipped::chars(r#"{ "path": "src/index.ts", "bytes": 49152, "encoding": "utf-8", "range": [0, 49152] }"#, 40) }
            ToolCall { name: "restart_service", state: Lifecycle::Denied, duration: ms(),
                error: rsx! { "Requires the " code { "deploy" } " permission." } }
            ToolCall { name: "long_job", state: Lifecycle::Aborted, duration: ms(), error: rsx! { "Stopped by you." } }
            ToolCall { name: "write_report", state: Lifecycle::Error, duration: ms(),
                error: rsx! { code { "EPERM" } ": the output path is not writable." } }
            ToolCall { name: "list_files", state: Lifecycle::Complete, duration: ms(), payload_label: rsx! { "Result" },
                payload: Clipped::full("README.md, Cargo.toml, src/") }
        }
    }
}

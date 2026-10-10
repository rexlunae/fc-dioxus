use fc_dioxus::prelude::*;

// example:start
/// Provider, model, and effort as one piece of state. Choose Local to see the effort
/// control dim with a note instead of disappearing; switch back and the model you had
/// under that provider returns.
#[component]
pub fn States() -> Element {
    let mut state = use_signal(|| None::<ContextState>);
    let model = |id: &str, name: &str, reasoning: bool, context: &str| ModelChoice {
        id: id.into(),
        name: name.into(),
        reasoning,
        parameters: vec![
            ("context".into(), context.into()),
            ("temp".into(), "0.7".into()),
        ],
    };
    let providers = vec![
        ProviderChoice {
            id: "anthropic".into(),
            name: "Anthropic".into(),
            models: vec![
                model("opus", "Opus", true, "1M"),
                model("sonnet", "Sonnet", true, "1M"),
                model("haiku", "Haiku", false, "200k"),
            ],
        },
        ProviderChoice {
            id: "local".into(),
            name: "Local".into(),
            models: vec![model("llama", "Llama", false, "128k")],
        },
    ];
    rsx! {
        ContextStrip { providers, onchange: move |s| state.set(Some(s)) }
        p { role: "status",
            if let Some(s) = state() {
                "{s.provider} · {s.model} · effort {s.effort.map_or(\"n/a\", |e| e.as_str())}"
            }
        }
    }
}

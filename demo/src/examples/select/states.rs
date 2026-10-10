use fc_dioxus::prelude::*;

// example:start
/// A grouped select with a value, an unavailable option, descriptions, and typeahead;
/// one showing its placeholder; and a disabled one. Focus stays on the button while the
/// list is open; Escape puts the previous value back.
#[component]
pub fn States() -> Element {
    const CLUSTER: Bem = Bem::new("fc-cluster");
    let mut model = use_signal(|| Some("opus".to_owned()));
    let models = vec![
        SelectChoice::new("opus", "Opus")
            .in_group("Anthropic")
            .describe("Most capable")
            .icon("✦")
            .meta("$$$"),
        SelectChoice::new("sonnet", "Sonnet")
            .in_group("Anthropic")
            .describe("Balanced"),
        SelectChoice::new("haiku", "Haiku")
            .in_group("Anthropic")
            .describe("Fastest")
            .meta("$"),
        SelectChoice::new("llama", "Llama (needs 64 GB)")
            .in_group("Local")
            .disabled(),
    ];
    let efforts = vec![
        SelectChoice::new("low", "Low"),
        SelectChoice::new("medium", "Medium"),
        SelectChoice::new("high", "High"),
    ];
    rsx! {
        div { class: CLUSTER.block(),
            Select { label: name!("Model"), options: models, value: ReadSignal::from(model), onchange: move |v| model.set(Some(v)) }
            Select { label: name!("Effort"), options: efforts, placeholder: "Select effort" }
            Select { label: name!("Region"), options: vec![SelectChoice::new("eu-west-1", "eu-west-1")],
                default_value: "eu-west-1".to_owned(), availability: Availability::Disabled }
            // With nothing to choose, the control is unavailable and its list says so.
            Select { label: name!("Workspace"), options: vec![], placeholder: "No workspaces" }
        }
        p { role: "status", "Model: {model().unwrap_or_default()}" }
    }
}

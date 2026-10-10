//! The context strip's rules, from upstream's context-strip.ts.
#![forbid(unsafe_code)]

use fc_dioxus::prelude::*;

fn catalog() -> Vec<ProviderChoice> {
    let model = |id: &str, reasoning: bool| ModelChoice {
        id: id.into(),
        name: id.into(),
        reasoning,
        parameters: vec![],
    };
    vec![
        ProviderChoice {
            id: "a".into(),
            name: "A".into(),
            models: vec![model("shared", true), model("a2", false)],
        },
        ProviderChoice {
            id: "b".into(),
            name: "B".into(),
            models: vec![model("b1", false), model("shared", true)],
        },
    ]
}

#[test]
fn provider_change_keeps_a_shared_model_and_restores_the_last_one() {
    let mut strip = fc_dioxus::behavior::context::ContextModel::new(catalog());
    assert_eq!(strip.state().model, "shared");
    assert_eq!(strip.state().effort, Some(Effort::Medium));
    strip.set_provider("b");
    assert_eq!(strip.state().model, "shared", "a model both offer is kept");
    strip.set_model("b1");
    strip.set_provider("a");
    assert_eq!(strip.state().model, "shared");
    strip.set_provider("b");
    assert_eq!(
        strip.state().model,
        "b1",
        "switching back restores that provider's model"
    );
}

#[test]
fn effort_applies_only_to_reasoning_models_and_is_kept_when_it_does_not() {
    let mut strip = fc_dioxus::behavior::context::ContextModel::new(catalog());
    strip.set_effort(Effort::High);
    strip.set_model("a2");
    assert_eq!(strip.state().effort, None);
    assert_eq!(strip.notice(), Some("Effort is not used by this model"));
    strip.set_effort(Effort::Low);
    strip.set_model("shared");
    assert_eq!(
        strip.state().effort,
        Some(Effort::High),
        "a set effort survives a model without one"
    );
    assert_eq!(strip.notice(), None);
}

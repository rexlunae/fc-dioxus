use fc_dioxus::prelude::*;

// example:start
/// Prompt cards in a grid, including disabled and unavailable ones, and follow-up pills.
#[component]
pub fn States() -> Element {
    const STACK: Bem = Bem::new("fc-stack");
    let mut chosen = use_signal(String::new);
    let reason = use_id();
    rsx! {
        div { class: STACK.block(),
            Suggestions { label: name!("Suggested prompts"), grid: true,
                Suggestion { hint: rsx! { "in five bullet points" }, onclick: move |_| chosen.set("Summarise this repository".into()), "Summarise this repository" }
                Suggestion { hint: rsx! { "needs permission" }, availability: Availability::Disabled, "Deploy to production" }
                Suggestion { availability: Availability::Unavailable { reason: reason.clone() }, "Open the dashboard" }
            }
            p { id: "{reason}", "The dashboard is offline for maintenance." }
            Suggestions { label: name!("Follow-ups"),
                Suggestion { pill: true, onclick: move |_| chosen.set("How big is it minified?".into()), "How big is it minified?" }
                Suggestion { pill: true, availability: Availability::Disabled, "Disabled follow-up" }
            }
            p { role: "status", if !chosen().is_empty() { "You picked: {chosen}" } }
        }
    }
}

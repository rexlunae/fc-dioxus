use fc_dioxus::prelude::*;

// example:start
/// Tones, counters, native toggle tags, removable tokens, and keyboard keys.
#[component]
pub fn States() -> Element {
    const CLUSTER: Bem = Bem::new("fc-cluster");
    const STACK: Bem = Bem::new("fc-stack");
    let mut rust = use_signal(|| false);
    let mut typescript = use_signal(|| true);
    let mut removed = use_signal(|| false);
    let reason = use_id();
    let Ok(label) = AccessibleName::try_from("Remove reviewer: ada") else {
        return rsx! {};
    };
    rsx! {
        div { class: STACK.block(),
            div { class: CLUSTER.block(),
                Badge { "Draft" }
                Badge { tone: BadgeTone::Accent, "Beta" }
                Badge { tone: BadgeTone::Success, dot: true, "Passing" }
                Badge { tone: BadgeTone::Warning, "Rate limited" }
                Badge { tone: BadgeTone::Danger, "Failed" }
                Badge { tone: BadgeTone::Info, "Info" }
                Badge { tone: BadgeTone::Accent, solid: true, "Solid" }
                Badge { tone: BadgeTone::Danger, solid: true, "Solid accent fill" }
                Badge { tone: BadgeTone::Info, outline: true, "Outline" }
                Badge { square: true, "Square" }
                Badge { tone: BadgeTone::Solid, count: true, aria_label: "3 unread", "3" }
                Badge { clipped: true, "A clipped label when space is constrained" }
            }
            div { class: CLUSTER.block(),
                Tag { pressed: typescript(), onchange: move |value| typescript.set(value), "TypeScript" }
                Tag { pressed: rust(), onchange: move |value| rust.set(value), "Rust" }
                Tag { availability: Availability::Disabled, "Go" }
                Tag { availability: Availability::Unavailable { reason: reason.clone() }, "Zig" }
                Tag { remove: TagRemove { label, onremove: EventHandler::new(move |()| removed.set(true)) }, "reviewer: ada" }
            }
            p { id: "{reason}", "Zig support is unavailable in this example." }
            p { role: "status", if removed() { "Removal requested for reviewer: ada." } }
            p { "Press " Kbd { "Ctrl" } " + " Kbd { "Enter" } " to send." }
        }
    }
}

use fc_dioxus::prelude::*;

// example:start
/// A single-choice toggle group, a small group with unavailable options, and actions.
#[component]
pub fn States() -> Element {
    const STACK: Bem = Bem::new("fc-stack");
    let mut align = use_signal(|| "start");
    let mut speed = use_signal(|| "1");
    let reason = use_id();
    let names = ["Alignment", "Speed", "Export"].map(AccessibleName::try_from);
    let [Ok(alignment), Ok(rate), Ok(export)] = names else {
        return rsx! {};
    };
    rsx! {
        div { class: STACK.block(),
            ButtonGroup { label: alignment,
                for (value, text) in [("start", "Start"), ("center", "Center"), ("end", "End")] {
                    Button { key: "{value}", variant: ButtonVariant::Secondary, pressed: align() == value, onclick: move |_| align.set(value), "{text}" }
                }
            }
            ButtonGroup { label: rate,
                Button { variant: ButtonVariant::Secondary, size: ButtonSize::Sm, pressed: speed() == "0.5", onclick: move |_| speed.set("0.5"), "0.5×" }
                Button { variant: ButtonVariant::Secondary, size: ButtonSize::Sm, pressed: speed() == "1", onclick: move |_| speed.set("1"), "1×" }
                Button { variant: ButtonVariant::Secondary, size: ButtonSize::Sm, availability: Availability::Disabled, "1.5×" }
                Button { variant: ButtonVariant::Secondary, size: ButtonSize::Sm, availability: Availability::Unavailable { reason: reason.clone() }, "2×" }
            }
            p { id: "{reason}", "2× playback needs a newer browser." }
            ButtonGroup { label: export,
                Button { variant: ButtonVariant::Secondary, "Export the whole conversation history" }
                Button { variant: ButtonVariant::Secondary, "Copy link" }
            }
        }
    }
}

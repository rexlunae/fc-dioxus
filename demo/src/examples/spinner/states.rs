use fc_dioxus::prelude::*;

// example:start
/// Every drawing, sizes and tones, an announced status, and shimmering text.
#[component]
pub fn States() -> Element {
    const CLUSTER: Bem = Bem::new("fc-cluster");
    const STACK: Bem = Bem::new("fc-stack");
    const MUTED: Bem = Bem::new("fc-u-text-muted");
    let Ok(typing) = AccessibleName::try_from("Assistant is typing") else {
        return rsx! {};
    };
    rsx! {
        div { class: STACK.block(),
            div { class: CLUSTER.block(),
                for kind in [SpinnerKind::Ring, SpinnerKind::Arc, SpinnerKind::Dots, SpinnerKind::Pulse,
                    SpinnerKind::Bars, SpinnerKind::Orbit, SpinnerKind::Sparkle]
                {
                    Spinner { kind, label: SpinnerLabel::Decorative }
                }
            }
            div { class: CLUSTER.block(),
                Spinner { size: SpinnerSize::Xs, label: SpinnerLabel::Decorative }
                Spinner { size: SpinnerSize::Sm, label: SpinnerLabel::Decorative }
                Spinner { size: SpinnerSize::Lg, label: SpinnerLabel::Decorative }
                Spinner { size: SpinnerSize::Xl, tone: SpinnerTone::Muted, label: SpinnerLabel::Decorative }
                span { class: MUTED.block(), "inline "
                    Spinner { size: SpinnerSize::Xs, tone: SpinnerTone::Current, label: SpinnerLabel::Decorative }
                    " current colour"
                }
            }
            div { class: CLUSTER.block(),
                Spinner { kind: SpinnerKind::Dots, label: SpinnerLabel::Status(typing) }
                ShimmerText { "Running tests" }
            }
        }
    }
}

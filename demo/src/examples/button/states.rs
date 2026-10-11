use fc_dioxus::prelude::*;

// example:start
/// Every button style, size, and availability state from the upstream fixture.
#[component]
pub fn States() -> Element {
    const CLUSTER: Bem = Bem::new("fc-cluster");
    const STACK: Bem = Bem::new("fc-stack");
    let mut activations = use_signal(|| 0_u32);
    let reason = use_id();
    let Ok(refresh) = AccessibleName::try_from("Refresh") else {
        return rsx! {};
    };
    rsx! {
        div { class: STACK.block(),
            div { class: CLUSTER.block(),
                for variant in [ButtonVariant::Primary, ButtonVariant::Secondary, ButtonVariant::Ghost, ButtonVariant::Tertiary, ButtonVariant::Ok, ButtonVariant::Cancel, ButtonVariant::Subtle, ButtonVariant::Danger] {
                    Button { variant, onclick: move |_| activations += 1, "{variant:?}" }
                }
            }
            div { class: CLUSTER.block(),
                for size in [ButtonSize::Sm, ButtonSize::Md, ButtonSize::Lg] {
                    Button { variant: ButtonVariant::Primary, size, onclick: move |_| activations += 1, "{size:?} action" }
                }
                Button { variant: ButtonVariant::Secondary, icon_label: refresh, onclick: move |_| activations += 1, Icon { name: IconName::Refresh } }
            }
            div { class: CLUSTER.block(),
                for variant in [ButtonVariant::Primary, ButtonVariant::Secondary, ButtonVariant::Ghost, ButtonVariant::Subtle, ButtonVariant::Danger] {
                    Button { variant, availability: Availability::Disabled, onclick: move |_| activations += 1, "Disabled" }
                }
            }
            div { class: CLUSTER.block(),
                for variant in [ButtonVariant::Primary, ButtonVariant::Secondary, ButtonVariant::Danger] {
                    Button { variant, availability: Availability::Unavailable { reason: reason.clone() }, onclick: move |_| activations += 1, "Unavailable" }
                }
            }
            p { id: "{reason}", "Unavailable actions stay focusable. Connect a workspace to enable them." }
            div { class: CLUSTER.block(),
                Button { variant: ButtonVariant::Primary, busy: true, onclick: move |_| activations += 1, "Saving…" }
                Button { variant: ButtonVariant::Secondary, busy: true, onclick: move |_| activations += 1, "Loading" }
            }
            Button { variant: ButtonVariant::Primary, block: true, onclick: move |_| activations += 1, "Full width action" }
            output { "data-activations": "", "Activated {activations} times" }
        }
    }
}

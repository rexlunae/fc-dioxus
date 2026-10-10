use fc_dioxus::prelude::*;

// example:start
/// A popover opened from a button, and tooltips above, below, and already dismissed.
/// Press Escape over a tooltip to hide it without moving away.
#[component]
pub fn States() -> Element {
    const CLUSTER: Bem = Bem::new("fc-cluster");
    let sampling = use_id();
    let names = ["Copy", "Share", "Pin"].map(AccessibleName::try_from);
    let [Ok(copy), Ok(share), Ok(pin)] = names else {
        return rsx! {};
    };
    rsx! {
        div { class: CLUSTER.block(),
            Button { variant: ButtonVariant::Secondary, popovertarget: "{sampling}", "Sampling" }
            Popover { id: sampling.clone(), title: rsx! { "Sampling" },
                footer: rsx! { Button { variant: ButtonVariant::Primary, size: ButtonSize::Sm, popovertarget: "{sampling}", popovertargetaction: "hide", "Done" } },
                p { "Temperature 0.7" }
            }
            Tooltip { tip: rsx! { "Copy to clipboard" }, Button { variant: ButtonVariant::Ghost, icon_label: copy, "⧉" } }
            Tooltip { tip: rsx! { "Share link" }, below: true, Button { variant: ButtonVariant::Ghost, icon_label: share, "↗" } }
            Tooltip { tip: rsx! { "Pin chat" }, dismissed: true, Button { variant: ButtonVariant::Ghost, icon_label: pin, "★" } }
        }
    }
}

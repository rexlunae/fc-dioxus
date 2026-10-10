use fc_dioxus::prelude::*;

// example:start
/// Brand, current and unavailable links, a disabled button, and busy global actions.
#[component]
pub fn States() -> Element {
    let reason = use_id();
    let Ok(label) = AccessibleName::try_from("Masthead fixture") else {
        return rsx! {};
    };
    rsx! {
        Masthead { href: "#masthead", mark: rsx! { "fc" }, brand: rsx! { "Workspace with a long name" }, nav_label: label,
            nav: rsx! {
                MastheadLink { href: "#masthead", current: true, "Current" }
                MastheadLink { href: "#masthead", "Rest / hover / active / focus" }
                MastheadLink { href: "#masthead", availability: Availability::Unavailable { reason: reason.clone() }, "Unavailable" }
                MastheadLink { onclick: move |_| {}, availability: Availability::Disabled, "Disabled" }
            },
            actions: rsx! {
                Button { variant: ButtonVariant::Primary, busy: true, "Saving…" }
                Badge { tone: BadgeTone::Danger, "Sync error" }
            },
        }
        p { id: "{reason}", "Billing opens after setup." }
    }
}

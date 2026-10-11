use fc_dioxus::prelude::*;

// example:start
/// Every icon, its sizes and RTL mirroring, and icon buttons in every state.
#[component]
pub fn States() -> Element {
    const CLUSTER: Bem = Bem::new("fc-cluster");
    const STACK: Bem = Bem::new("fc-stack");
    let mut copied = use_signal(|| false);
    let reason = use_id();
    let names = ["Copy", "Delete", "Refresh", "passed", "failed"].map(AccessibleName::try_from);
    let [Ok(copy), Ok(delete), Ok(refresh), Ok(passed), Ok(failed)] = names else {
        return rsx! {};
    };
    rsx! {
        div { class: STACK.block(),
            div { class: CLUSTER.block(),
                for name in IconName::ALL {
                    span { key: "{name.as_str()}", class: CLUSTER.block(), Icon { name: *name } code { "{name.as_str()}" } }
                }
            }
            div { class: CLUSTER.block(),
                for size in [IconSize::Sm, IconSize::Md, IconSize::Lg] {
                    span { Icon { name: IconName::Star, size } " {size:?}" }
                }
            }
            // Arrows that point along the text flip in RTL; play and refresh never do.
            div { class: CLUSTER.block(),
                for name in [IconName::ArrowStart, IconName::ChevronEnd, IconName::Send, IconName::Undo] { Icon { name } }
                span { dir: "rtl",
                    for name in [IconName::ArrowStart, IconName::ChevronEnd, IconName::Send, IconName::Undo] { Icon { name } }
                }
            }
            p { "Build " Icon { name: IconName::Success, label: passed } " lint " Icon { name: IconName::Error, label: failed } }
            div { class: CLUSTER.block(),
                for variant in [ButtonVariant::Primary, ButtonVariant::Secondary, ButtonVariant::Ghost, ButtonVariant::Subtle, ButtonVariant::Danger] {
                    IconButton { variant, icon: IconName::Copy, label: copy.clone() }
                }
            }
            div { class: CLUSTER.block(),
                for size in [ButtonSize::Sm, ButtonSize::Md, ButtonSize::Lg] {
                    IconButton { variant: ButtonVariant::Ghost, size, icon: IconName::Copy, label: copy.clone() }
                }
                // The icon confirms the copy; the name stays "Copy" so it is not
                // re-announced differently under focus.
                IconButton { variant: ButtonVariant::Secondary, label: copy.clone(),
                    icon: if copied() { IconName::Check } else { IconName::Copy },
                    onclick: move |_| copied.toggle() }
            }
            div { class: CLUSTER.block(),
                IconButton { variant: ButtonVariant::Secondary, icon: IconName::Trash, label: delete.clone(), availability: Availability::Disabled }
                IconButton { variant: ButtonVariant::Secondary, icon: IconName::Trash, label: delete, availability: Availability::Unavailable { reason: reason.clone() } }
                IconButton { variant: ButtonVariant::Secondary, icon: IconName::Refresh, label: refresh, busy: true }
                Button { variant: ButtonVariant::Secondary, busy: true, "Refreshing" }
            }
            p { id: "{reason}", "Deleting needs edit access to this project." }
            div { class: CLUSTER.block(),
                Button { variant: ButtonVariant::Primary, Icon { name: IconName::Download } span { "Download" } }
                Button { variant: ButtonVariant::Secondary, Icon { name: IconName::ExternalLink } span { "Open in a new tab" } }
            }
        }
    }
}

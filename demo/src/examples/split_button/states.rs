use fc_dioxus::prelude::*;

// example:start
/// Variants and sizes, an open menu, and disabled and busy groups.
#[component]
pub fn States() -> Element {
    const CLUSTER: Bem = Bem::new("fc-cluster");
    let mut open = use_signal(|| true);
    let names = [
        "Send",
        "More send options",
        "Merge",
        "More merge options",
        "Export",
        "More export options",
        "Run",
        "More run options",
        "Deploy",
        "More deploy options",
        "Save",
        "More save options",
    ]
    .map(AccessibleName::try_from);
    let [Ok(send), Ok(send_more), Ok(merge), Ok(merge_more), Ok(export), Ok(export_more), Ok(run), Ok(run_more), Ok(deploy), Ok(deploy_more), Ok(save), Ok(save_more)] =
        names
    else {
        return rsx! {};
    };
    rsx! {
        div { class: CLUSTER.block(),
            SplitButton { label: send, menu_label: send_more, variant: ButtonVariant::Primary, "Send" }
            SplitButton { label: merge, menu_label: merge_more, variant: ButtonVariant::Secondary, "Merge" }
            SplitButton { label: export, menu_label: export_more, variant: ButtonVariant::Subtle, size: ButtonSize::Sm, "Export" }
        }
        div { class: CLUSTER.block(),
            SplitButton { label: run, menu_label: run_more, variant: ButtonVariant::Primary,
                expanded: open(), ontoggle: move |()| open.toggle(), "Run" }
            SplitButton { label: deploy, menu_label: deploy_more, variant: ButtonVariant::Primary, availability: Availability::Disabled, "Deploy" }
            SplitButton { label: save, menu_label: save_more, variant: ButtonVariant::Secondary, busy: true, "Saving…" }
        }
    }
}

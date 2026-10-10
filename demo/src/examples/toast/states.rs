use fc_dioxus::prelude::*;

// example:start
/// Every tone and a toast with an action, drawn statically, then a live toaster: danger
/// toasts stay and are announced assertively, the rest time out unless you hover or focus
/// the stack.
#[component]
pub fn States() -> Element {
    const STACK: Bem = Bem::new("fc-stack");
    let Ok(top_label) = AccessibleName::try_from("Top notifications") else {
        return rsx! {};
    };
    rsx! {
        div { class: STACK.block(),
            ToastList {
                Toast { tone: ToastTone::Info, title: "Indexing", body: "Finished 412 files.", ondismiss: move |()| {} }
                Toast { tone: ToastTone::Success, title: "Saved", body: "Settings saved.", ondismiss: move |()| {} }
                Toast { tone: ToastTone::Warning, title: "Rate limited", body: "Retrying in 30s.", ondismiss: move |()| {} }
                Toast { tone: ToastTone::Danger, title: "Upload failed", body: "The file is over 25 MB.", ondismiss: move |()| {} }
            }
            ToastList {
                Toast { title: "Chat archived", body: "Moved to the archive.", ondismiss: move |()| {},
                    action: ToastAction { label: "Undo".into(), onaction: EventHandler::new(move |()| {}) } }
            }
            Toaster { LiveToasts {} }
            // A second toaster, centred at the top for layouts whose corner holds a composer.
            Toaster { top: true, label: top_label, TopToasts {} }
        }
    }
}

/// Buttons that raise toasts through the nearest toaster.
#[component]
fn LiveToasts() -> Element {
    const CLUSTER: Bem = Bem::new("fc-cluster");
    let toasts = use_toasts();
    let mut undone = use_signal(|| 0);
    rsx! {
        div { class: CLUSTER.block(),
            Button { variant: ButtonVariant::Secondary,
                onclick: move |_| { toasts.show(ToastOptions { tone: ToastTone::Success, title: Some("Saved".into()), body: Some("Settings saved.".into()), ..ToastOptions::default() }); },
                "Show a success toast" }
            Button { variant: ButtonVariant::Secondary,
                onclick: move |_| { toasts.show(ToastOptions { title: Some("Chat archived".into()),
                    action: Some(ToastAction { label: "Undo".into(), onaction: EventHandler::new(move |()| undone += 1) }), ..ToastOptions::default() }); },
                "Archive with undo" }
            Button { variant: ButtonVariant::Danger,
                onclick: move |_| { toasts.show(ToastOptions { tone: ToastTone::Danger, title: Some("Upload failed".into()), body: Some("The file is over 25 MB.".into()), ..ToastOptions::default() }); },
                "Show an error toast" }
        }
        p { role: "status", if undone() > 0 { "Undone {undone} time(s)." } }
    }
}

/// One button raising a toast in the top-centred toaster.
#[component]
fn TopToasts() -> Element {
    let toasts = use_toasts();
    rsx! {
        Button { variant: ButtonVariant::Secondary,
            onclick: move |_| { toasts.show(ToastOptions { tone: ToastTone::Info, body: Some("Reconnected.".into()), ..ToastOptions::default() }); },
            "Show at the top" }
    }
}

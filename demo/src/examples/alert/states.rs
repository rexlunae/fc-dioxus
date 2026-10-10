use fc_dioxus::prelude::*;

// example:start
/// All alert tones, named dismiss actions, and the edge-to-edge banner.
#[component]
pub fn States() -> Element {
    const STACK: Bem = Bem::new("fc-stack");
    let mut dismissed = use_signal(|| false);
    let Ok(label) = AccessibleName::try_from("Dismiss warning") else {
        return rsx! {};
    };
    rsx! {
        div { class: STACK.block(),
            Alert { tone: AlertTone::Info, title: rsx! { "New models available" }, "Pick one from the context strip." }
            Alert { tone: AlertTone::Success, "Settings saved." }
            Alert { tone: AlertTone::Warning, urgency: Urgency::Polite,
                title: rsx! { "Context window 92% full" },
                actions: rsx! { Button { variant: ButtonVariant::Secondary, size: ButtonSize::Sm, onclick: move |_| dismissed.set(false), "Start a new chat" } },
                dismiss: Dismiss { label, ondismiss: EventHandler::new(move |()| dismissed.set(true)) },
                if dismissed() { "Warning acknowledged. Start a new chat to reset this example." }
                else { "Older messages will be summarised on the next turn." }
            }
            Alert { tone: AlertTone::Danger, title: rsx! { "The tool call failed" }, "The sandbox ran out of memory after 41s." }
            Alert { tone: AlertTone::Info, banner: true, "Scheduled maintenance tonight at 22:00 UTC." }
            Alert { icon: AlertIcon::None, "Neutral feedback without an icon." }
        }
    }
}

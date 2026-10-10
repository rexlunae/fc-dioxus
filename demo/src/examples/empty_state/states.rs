use fc_dioxus::prelude::*;

// example:start
/// Nothing yet, a failure the user caused (so it is announced), and the compact form.
#[component]
pub fn States() -> Element {
    const STACK: Bem = Bem::new("fc-stack");
    let mut retries = use_signal(|| 0);
    rsx! {
        div { class: STACK.block(),
            EmptyState {
                icon: rsx! { "☰" },
                title: rsx! { "No chats yet" },
                body: rsx! { "Start one from the composer." },
                actions: rsx! { Button { variant: ButtonVariant::Primary, "New chat" } },
            }
            EmptyState { error: true, announce: true,
                icon: rsx! { "!" },
                title: rsx! { "Could not load chats" },
                body: rsx! { "The server did not respond. Retries: {retries}" },
                actions: rsx! { Button { variant: ButtonVariant::Secondary, onclick: move |_| retries += 1, "Retry" } },
            }
            EmptyState { compact: true, body: rsx! { "No results for “zzz”." } }
        }
    }
}

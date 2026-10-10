use fc_dioxus::prelude::*;

// example:start
/// A submitting search with recent queries, a docked suggestion view while loading, and
/// disabled and invalid bars.
#[component]
pub fn States() -> Element {
    const STACK: Bem = Bem::new("fc-stack");
    let mut submitted = use_signal(String::new);
    let mut models = use_signal(|| "sonnet".to_owned());
    let names = [
        "Search conversations",
        "Recent searches",
        "Search by voice",
        "Search files",
        "Suggestions",
        "Search (offline)",
        "Search by date",
        "Filter models",
        "Clear search",
    ]
    .map(AccessibleName::try_from);
    let [Ok(conversations), Ok(recent), Ok(voice), Ok(files), Ok(suggestions), Ok(offline), Ok(by_date), Ok(filter), Ok(clear)] =
        names
    else {
        return rsx! {};
    };
    rsx! {
        div { class: STACK.block(),
            Search { label: conversations, placeholder: "Search conversations",
                onsubmit: move |query| submitted.set(query),
                trailing: rsx! { Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, icon_label: voice, "🎤" } },
                view: rsx! {
                    SearchView { label: recent, heading: rsx! { "Recent" },
                        List { ListItem { headline: rsx! { "deploy checklist" }, action: ListAction::Link("#q".into()) } }
                    }
                },
            }
            p { role: "status", if !submitted().is_empty() { "Searched for “{submitted}”." } }
            Search { label: files, default_value: "quarterly", onsubmit: move |_| {},
                view: rsx! {
                    SearchView { label: suggestions, heading: rsx! { "Suggestions" }, docked: true, open: true, busy: true,
                        List {
                            ListItem { headline: rsx! { "Quarterly plan" }, supporting: rsx! { "Shared with you" }, action: ListAction::Link("#q1".into()) }
                            ListItem { headline: rsx! { "Quarterly review notes with an unusually long file name" }, action: ListAction::Link("#q2".into()) }
                        }
                    }
                },
            }
            Search { label: offline, placeholder: "Offline", availability: Availability::Disabled }
            Search { label: by_date, default_value: "2026-13-40", invalid: true }
            // A named clear button replaces the browser's own, so there are never two.
            Search { label: filter, value: ReadSignal::from(models), oninput: move |value| models.set(value), own_clear: true,
                trailing: rsx! { Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, icon_label: clear, onclick: move |_| models.set(String::new()), "×" } } }
        }
    }
}

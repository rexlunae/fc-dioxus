use super::super::{long_path, record};
use fc_dioxus::{core::assets, prelude::*};

// example:start
/// A tool permission request in every lifecycle state, with the exact arguments one
/// disclosure away. Deny comes first, so allowing is never the default by position.
#[component]
pub fn States() -> Element {
    const GRID: Bem = Bem::new("fc-grid-auto");
    const CODE: Bem = Bem::new("fc-code");
    const SOURCE: Bem = Bem::new("fc-code__source");
    const DISCLOSURE: Bem = Bem::new("fc-disclosure");
    const SUMMARY: Bem = Bem::new("fc-disclosure__summary");
    let mut decision = use_signal(|| None::<bool>);
    rsx! {
        // The code block's own component is still to come; load its stylesheet directly.
        document::Stylesheet { href: assets::COMPONENTS_CODE_CSS }
        div { class: GRID.block(),
            for (index, state) in Lifecycle::ALL.iter().copied().enumerate() {
                Approval { key: "{index}", label: name!("Allow read_file?"), record: record("approval", state, index),
                    status: rsx! { p { "{state:?}" } }, title: rsx! { "Allow read_file?" },
                    actions: rsx! {
                        Button { variant: ButtonVariant::Secondary, availability: Availability::Disabled, "Deny" }
                        Button { variant: ButtonVariant::Primary, availability: Availability::Disabled, "Allow once" }
                    },
                    p { "Scope: the selected project, once." }
                    // Upstream's approval markup puts the code directly in the details, without
                    // the Disclosure content wrapper.
                    details { class: DISCLOSURE.block(),
                        summary { class: SUMMARY.block(), "Full arguments" }
                        div { class: CODE.block(), pre { class: SOURCE.block(), code { r#"{{"path":"README.md"}}"# } } }
                    }
                }
            }
        }
        div { class: GRID.block(),
            Approval { label: name!("Inspect the full request"), region: true, record: record("approval", Lifecycle::Pending, 100),
                title: rsx! { "Inspect the full request before deciding" },
                details { class: DISCLOSURE.block(), open: true,
                    summary { class: SUMMARY.block(), "Full arguments" }
                    div { class: CODE.block(), pre { class: SOURCE.block(), code { r#"{{"path":"{long_path()}"}}"# } } }
                }
            }
            Approval { label: name!("No approval requests"), region: true, record: record("approval", Lifecycle::Complete, 101),
                title: rsx! { "No approval requests" }, p { "Nothing is waiting for a decision." } }
        }
        Approval { label: name!("Allow web_search?"), title: rsx! { "Allow web_search?" },
            record: Record { state: Some(match decision() { None => Lifecycle::Pending, Some(true) => Lifecycle::Complete, Some(false) => Lifecycle::Denied }), ..Record::default() },
            meta: rsx! { match decision() { None => "Waiting for you.", Some(true) => "Allowed once.", Some(false) => "Denied." } },
            actions: rsx! {
                Button { variant: ButtonVariant::Secondary, availability: if decision().is_some() { Availability::Disabled } else { Availability::Enabled }, onclick: move |_| decision.set(Some(false)), "Deny" }
                Button { variant: ButtonVariant::Primary, availability: if decision().is_some() { Availability::Disabled } else { Availability::Enabled }, onclick: move |_| decision.set(Some(true)), "Allow once" }
            },
            p { "Search the web for the latest release notes." }
        }
    }
}

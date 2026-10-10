use fc_dioxus::prelude::*;

// example:start
/// Native checkboxes in a disclosure, with a filter, bulk actions that skip unavailable
/// people, and a live summary. Escape closes it.
#[component]
pub fn States() -> Element {
    let mut members = use_signal(|| vec!["ada".to_owned()]);
    let option =
        |value: &str, title: &str, description: &str, availability: Availability| MultiOption {
            value: value.into(),
            title: title.into(),
            description: Some(description.into()),
            availability,
        };
    rsx! {
        MultiSelect { label: rsx! { "Project members" }, legend: rsx! { "Available members" }, name: "members", open: true,
            values: ReadSignal::from(members), onchange: move |v| members.set(v),
            filter_label: "Find members", filter_placeholder: "Search names or roles",
            options: vec![
                option("ada", "Ada Lovelace", "Maintainer · Design systems", Availability::Enabled),
                option("grace", "Grace Hopper", "Reviewer · Compilers", Availability::Enabled),
                option("alan", "Alan Turing", "Unavailable · Access pending", Availability::Disabled),
            ],
        }
        p { role: "status", "Chosen: {members().join(\", \")}" }
    }
}

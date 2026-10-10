use fc_dioxus::prelude::*;

// example:start
/// Every checkbox state, an inline radio group, switch sizes, and a switch row.
#[component]
pub fn States() -> Element {
    const STACK: Bem = Bem::new("fc-stack");
    const CLUSTER: Bem = Bem::new("fc-cluster");
    let density_name = use_id();
    let mut density = use_signal(|| "comfortable".to_owned());
    let mut mixed = use_signal(|| CheckState::Mixed);
    let names =
        ["Small, off", "On", "Large, on", "Disabled", "Disabled, on"].map(AccessibleName::try_from);
    let [Ok(small), Ok(on), Ok(large), Ok(disabled), Ok(disabled_on)] = names else {
        return rsx! {};
    };
    rsx! {
        div { class: STACK.block(),
            CheckGroup { legend: rsx! { "Notify me" },
                Checkbox { label: rsx! { "Unchecked" } }
                Checkbox { label: rsx! { "Checked" }, default_checked: CheckState::On }
                Checkbox { label: rsx! { "Indeterminate" }, checked: ReadSignal::from(mixed),
                    onchange: move |on| mixed.set(if on { CheckState::On } else { CheckState::Off }) }
                Checkbox { label: rsx! { "Disabled" }, availability: Availability::Disabled }
                Checkbox { label: rsx! { "Checked, disabled" }, default_checked: CheckState::On, availability: Availability::Disabled }
                Checkbox { label: rsx! { "Invalid" }, invalid: true, hint: rsx! { "You must accept the terms." } }
            }
            CheckGroup { legend: rsx! { "Density" }, inline: true,
                radios: RadioChoice { name: density_name, value: Some(density()), onchange: Some(EventHandler::new(move |value| density.set(value))) },
                Radio { label: rsx! { "Comfortable" }, value: "comfortable" }
                Radio { label: rsx! { "Compact" }, value: "compact" }
                Radio { label: rsx! { "Dense" }, value: "dense", availability: Availability::Disabled }
            }
            div { class: CLUSTER.block(),
                Switch { label: small, size: SwitchSize::Sm }
                Switch { label: on, default_checked: true }
                Switch { label: large, size: SwitchSize::Lg, default_checked: true }
                Switch { label: disabled, availability: Availability::Disabled }
                Switch { label: disabled_on, default_checked: true, availability: Availability::Disabled }
            }
            SwitchRow { label: rsx! { "Show reasoning by default" }, hint: rsx! { "Expands the thinking trace on every answer." }, default_checked: true }
        }
    }
}

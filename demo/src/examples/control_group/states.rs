use fc_dioxus::prelude::*;

// example:start
/// A joined search box, wrapping toggles, a fieldset that disables everything, and a
/// plain status beside its action.
#[component]
pub fn States() -> Element {
    const STACK: Bem = Bem::new("fc-stack");
    const LABEL: Bem = Bem::new("fc-field__label");
    let mut query = use_signal(|| "Projects".to_owned());
    let mut web = use_signal(|| true);
    let model = use_id();
    let query_id = use_id();
    let reason = use_id();
    let names =
        ["Search actions", "Search", "Tool choices", "Recovery"].map(AccessibleName::try_from);
    let [Ok(actions), Ok(search), Ok(tools), Ok(recovery)] = names else {
        return rsx! {};
    };
    rsx! {
        div { class: STACK.block(),
            label { class: LABEL.block(), r#for: "{query_id}", "{search}" }
            ControlGroup { name: GroupName::Label(actions), layout: ControlLayout::Joined,
                Button { variant: ButtonVariant::Secondary, onclick: move |_| query.set(String::new()), "Clear" }
                Input { id: "{query_id}", value: ReadSignal::from(query), oninput: move |value| query.set(value) }
                Button { variant: ButtonVariant::Primary, "Search" }
            }
            ControlGroup { name: GroupName::Label(tools),
                Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, pressed: web(), onclick: move |_| web.toggle(), "Web search" }
                Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, pressed: false, "An extremely long tool name that must wrap instead of widening the page" }
                Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, availability: Availability::Disabled, "Disabled" }
                Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, availability: Availability::Unavailable { reason: reason.clone() }, "Unavailable" }
                Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, busy: true, "Loading" }
            }
            p { id: "{reason}", "This tool is not enabled for your workspace." }
            ControlGroup { name: GroupName::Legend(rsx! { "Settings" }), layout: ControlLayout::Vertical, disabled: true,
                ControlLabel { r#for: model.clone(), "Model" }
                NativeSelect { id: "{model}", option { "Balanced" } }
                Button { variant: ButtonVariant::Secondary, "Apply" }
            }
            ControlGroup { name: GroupName::Label(recovery), layout: ControlLayout::Plain,
                Badge { tone: BadgeTone::Danger, "Export failed" }
                Button { variant: ButtonVariant::Secondary, "Retry" }
            }
        }
    }
}

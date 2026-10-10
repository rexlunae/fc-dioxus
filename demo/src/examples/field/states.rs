use fc_dioxus::prelude::*;

// example:start
/// Labelled fields in every state, sizes and surfaces, input groups, choice cards, and an
/// error summary. Each control is wired to its label, hint, and error by its `Field`.
#[component]
pub fn States() -> Element {
    const STACK: Bem = Bem::new("fc-stack");
    const GRID: Bem = Bem::new("fc-grid-auto");
    let mut email = use_signal(|| "ada@".to_owned());
    let mut visibility = use_signal(|| "private".to_owned());
    let visibility_name = use_id();
    let email_field = use_id();
    let invalid_email = !email().contains('.');
    let names = [
        "Small input",
        "Large input",
        "Model",
        "Disabled select",
        "System prompt",
        "Budget",
        "Domain",
    ]
    .map(AccessibleName::try_from);
    let [Ok(small), Ok(large), Ok(model), Ok(locked), Ok(prompt), Ok(budget), Ok(domain)] = names
    else {
        return rsx! {};
    };
    rsx! {
        div { class: STACK.block(),
            div { class: GRID.block(),
                Field { label: rsx! { "Display name" }, necessity: Necessity::Optional, hint: rsx! { "Shown on your messages." },
                    Input { default_value: "Ada" }
                }
                Field { label: rsx! { "Email" }, necessity: Necessity::Required,
                    Input { r#type: InputType::Email, placeholder: "ada@example.com" }
                }
                Field { label: rsx! { "Email" }, control: email_field.clone(),
                    error: invalid_email.then(|| rsx! { "Enter an email address, like ada@example.com" }),
                    Input { r#type: InputType::Email, value: ReadSignal::from(email), oninput: move |value| email.set(value) }
                }
            }
            div { class: GRID.block(),
                Field { label: rsx! { "Project name" }, Input { variant: InputVariant::Filled, default_value: "Atlas" } }
                Field { label: rsx! { "Budget" }, Input { variant: InputVariant::Filled, default_value: "-4", invalid: true } }
                Field { label: rsx! { "Owner" }, Input { variant: InputVariant::Filled, default_value: "Locked", availability: Availability::Disabled } }
                Field { label: rsx! { "Workspace id" }, Input { default_value: "ws_01H8", availability: Availability::Disabled } }
                Field { label: rsx! { "Plan" }, Input { default_value: "Team", readonly: true } }
            }
            div { class: GRID.block(),
                Input { size: InputSize::Sm, aria_label: "{small}", default_value: "Small" }
                Input { size: InputSize::Lg, aria_label: "{large}", default_value: "Large" }
                NativeSelect { aria_label: "{model}", option { "Balanced" } option { "Fast" } }
                NativeSelect { aria_label: "{locked}", availability: Availability::Disabled, option { "Unavailable" } }
                Textarea { autosize: true, aria_label: "{prompt}", default_value: "Grows as you type." }
            }
            div { class: GRID.block(),
                Field { label: rsx! { "Soft search" }, Input { variant: InputVariant::Soft, r#type: InputType::Search, placeholder: "Search projects" } }
                Field { label: rsx! { "Underlined title" }, Input { variant: InputVariant::Underlined, placeholder: "Untitled document" } }
                Field { label: rsx! { "Compact timezone" }, inline: true,
                    NativeSelect { size: InputSize::Sm, option { "Pacific time" } option { "UTC" } }
                }
            }
            div { class: GRID.block(),
                InputGroup {
                    InputAddon { decorative: true, "$" }
                    Input { aria_label: "{budget}", inputmode: "decimal", default_value: "40" }
                    InputAddon { "USD" }
                }
                InputGroup {
                    InputAddon { "https://" }
                    Input { aria_label: "{domain}", invalid: true, default_value: "exa mple.com" }
                }
            }
            CheckGroup { legend: rsx! { "Workspace visibility" },
                radios: RadioChoice { name: visibility_name, value: Some(visibility()), onchange: Some(EventHandler::new(move |value| visibility.set(value))) },
                Choice { value: "private", title: rsx! { "Private" }, description: rsx! { "Only invited members can view this workspace." } }
                Choice { value: "team", title: rsx! { "Team" }, description: rsx! { "Everyone in your organization can join." } }
                Choice { value: "public", availability: Availability::Disabled, title: rsx! { "Public · unavailable" },
                    description: rsx! { "Your plan does not include public workspaces." } }
            }
            if invalid_email {
                ErrorSummary { title: rsx! { "There is a problem" }, heading: HeadingLevel::H3,
                    errors: vec![FieldError { field: email_field, message: "Enter an email address".into() }] }
            }
        }
    }
}

use fc_dioxus::prelude::*;

// example:start
/// Static rows, navigation links, settings rows that label a control, and button rows.
#[component]
pub fn States() -> Element {
    const STACK: Bem = Bem::new("fc-stack");
    let mut search = use_signal(|| true);
    let search_value = ReadSignal::from(search);
    let mut selected = use_signal(|| 1);
    let reason = use_id();
    let Ok(delete) = AccessibleName::try_from("Delete draft") else {
        return rsx! {};
    };
    rsx! {
        div { class: STACK.block(),
            List { divided: true,
                ListItem { headline: rsx! { "One line" } }
                ListItem { headline: rsx! { "Ada Byron" }, supporting: rsx! { "ada@example.com" },
                    leading: rsx! { Avatar { name: "Ada Byron", alt: Alt::Decorative, size: AvatarSize::Sm } },
                    trailing: rsx! { "2h" } }
                ListItem { headline: rsx! { "Quarterly plan" }, overline: rsx! { "Shared with you" }, leading: rsx! { "📄" },
                    supporting_clipped: Clipped::lines("A long supporting line that runs past two lines and is clamped, with the clamp declared on the element so a machine reader knows the visible text is not the whole value.", 2) }
            }
            List {
                ListItem { headline: rsx! { "General" }, supporting: rsx! { "Language, appearance" }, leading: rsx! { "⚙" },
                    action: ListAction::Link("#general".into()), trailing: rsx! { "›" }, trailing_decorative: true }
                ListItem { headline: rsx! { "Privacy" }, supporting: rsx! { "Current page" }, leading: rsx! { "🔒" },
                    action: ListAction::Link("#privacy".into()), current: true }
                ListItem { headline: rsx! { "Billing" }, supporting: rsx! { "Unavailable on this plan" }, leading: rsx! { "💳" },
                    action: ListAction::Link("#billing".into()), availability: Availability::Unavailable { reason: reason.clone() } }
            }
            p { id: "{reason}", "Billing is managed by your organization's administrator." }
            List {
                // The row's <label> names each control, so the controls take no name of their own.
                ListItem { headline: rsx! { "Web search" }, supporting: rsx! { "Let the assistant search the web" }, action: ListAction::Label,
                    trailing: rsx! { Switch { label: ControlName::Wrapped, checked: search_value, onchange: move |on| search.set(on) } } }
                ListItem { headline: rsx! { "Include attachments" }, action: ListAction::Label,
                    trailing: rsx! { input { r#type: "checkbox" } } }
                ListItem { headline: rsx! { "Code execution" }, supporting: rsx! { "Disabled by your administrator" }, action: ListAction::Label,
                    trailing: rsx! { Switch { label: ControlName::Wrapped, availability: Availability::Disabled } } }
            }
            List {
                ListItem { headline: rsx! { "Draft reply to the infrastructure review thread with a long title" }, supporting: rsx! { "Yesterday" },
                    action: ListAction::Button(EventHandler::new(move |()| selected.set(0))), current: selected() == 0,
                    trailing: rsx! { Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, icon_label: delete.clone(), "×" } } }
                ListItem { headline: rsx! { "Selected draft" },
                    action: ListAction::Button(EventHandler::new(move |()| selected.set(1))), current: selected() == 1 }
            }
        }
    }
}

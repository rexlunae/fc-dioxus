use fc_dioxus::prelude::*;

/// Which example dialog is open.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Open {
    None,
    Delete,
    Sheet,
    Bottom,
    Fullscreen,
    Notice,
    Confirm,
    Create,
    Settings,
}

// example:start
/// Modal dialogs in the top layer: a confirmation, a side sheet, a bottom sheet, a
/// fullscreen view, and the notice, confirm, form, and settings recipes.
#[component]
pub fn States() -> Element {
    const CLUSTER: Bem = Bem::new("fc-cluster");
    const STACK: Bem = Bem::new("fc-stack");
    const FIELD: Bem = Bem::new("fc-field");
    const LABEL: Bem = Bem::new("fc-field__label");
    const INLINE: Bem = Bem::new("fc-field--inline");
    const INPUT: Bem = Bem::new("fc-input");
    const SOFT: Bem = Bem::new("fc-input--soft");
    const ALERT: Bem = Bem::new("fc-alert");
    const SUCCESS: Bem = Bem::new("fc-alert--success");
    const DESCRIPTION: Bem = Bem::new("fc-dialog__description");
    let mut open = use_signal(|| Open::None);
    let mut style = use_signal(|| "concise".to_owned());
    let mut result = use_signal(String::new);
    let style_name = use_id();
    let names = [
        "Close run inspector",
        "Close information",
        "Close confirmation",
        "Close form dialog",
        "Close preferences",
    ]
    .map(AccessibleName::try_from);
    let [Ok(close_inspector), Ok(close_notice), Ok(close_confirm), Ok(close_form), Ok(close_prefs)] =
        names
    else {
        return rsx! {};
    };
    let ids = [
        "fx-recipe-notice",
        "fx-recipe-confirm",
        "fx-recipe-create",
        "fx-recipe-settings",
        "fx-recipe-create-form",
        "fx-recipe-settings-form",
    ]
    .map(Id::try_from);
    let [Ok(notice_id), Ok(confirm_id), Ok(create_id), Ok(settings_id), Ok(create_form), Ok(settings_form)] =
        ids
    else {
        return rsx! {};
    };
    let close = move |()| open.set(Open::None);
    let show = |which: Open| move |_| open.set(which);
    rsx! {
        div { class: STACK.block(),
            div { class: CLUSTER.block(),
                Button { variant: ButtonVariant::Danger, onclick: show(Open::Delete), "Delete chat…" }
                Button { variant: ButtonVariant::Secondary, onclick: show(Open::Sheet), "Open inspector" }
                Button { variant: ButtonVariant::Secondary, onclick: show(Open::Bottom), "Share…" }
                Button { variant: ButtonVariant::Secondary, onclick: show(Open::Fullscreen), "Full screen" }
            }
            div { class: CLUSTER.block(),
                Button { onclick: show(Open::Notice), "Notice" }
                Button { onclick: show(Open::Confirm), "Confirm" }
                Button { onclick: show(Open::Create), "Form" }
                Button { onclick: show(Open::Settings), "Settings" }
            }
            p { role: "status", "{result}" }
        }
        Dialog { open: open() == Open::Delete, onclose: close, size: DialogSize::Sm, heading: HeadingLevel::H3,
            title: rsx! { "Delete this chat?" },
            footer: rsx! {
                Button { variant: ButtonVariant::Secondary, onclick: move |_| open.set(Open::None), "Cancel" }
                Button { variant: ButtonVariant::Danger, onclick: move |_| { result.set("Chat deleted.".into()); open.set(Open::None); }, "Delete" }
            },
            p { class: DESCRIPTION.block(), "This cannot be undone." }
        }
        Dialog { open: open() == Open::Sheet, onclose: close, placement: DialogPlacement::Sheet, heading: HeadingLevel::H3,
            close_label: close_inspector, title: rsx! { "Run inspector" },
            p { "Docks to the inline end." }
        }
        Dialog { open: open() == Open::Bottom, onclose: close, placement: DialogPlacement::Bottom, handle: true, heading: HeadingLevel::H3,
            title: rsx! { "Share conversation" },
            p { "Anyone with the link can read it." }
        }
        Dialog { open: open() == Open::Fullscreen, onclose: close, size: DialogSize::Fullscreen, title: rsx! { "Transcript" },
            p { "The whole conversation, with nothing else on screen." }
        }
        Dialog { open: open() == Open::Notice, onclose: close, size: DialogSize::Sm, heading: HeadingLevel::H3, fc_id: notice_id,
            close_label: close_notice, scrollable: true, title: rsx! { "Workspace ready" },
            description: rsx! { "Your workspace is ready. Invite teammates or start a new conversation." },
            footer: rsx! { Button { variant: ButtonVariant::Ok, value: "ok", onclick: move |_| open.set(Open::None), "OK" } },
            // A static confirmation; it is not announced, since the dialog itself is.
            div { class: "{ALERT.block()} {SUCCESS.block()}", span { aria_hidden: "true", "✓" } div { "Your preferences are saved locally for this demo." } }
        }
        Dialog { open: open() == Open::Confirm, onclose: close, size: DialogSize::Sm, heading: HeadingLevel::H3, fc_id: confirm_id,
            close_label: close_confirm, scrollable: true, title: rsx! { "Archive this project?" },
            description: rsx! { "The project will leave your active list. You can restore it from the archive." },
            footer: rsx! {
                Button { variant: ButtonVariant::Cancel, value: "cancel", onclick: move |_| open.set(Open::None), "Cancel" }
                Button { variant: ButtonVariant::Primary, value: "confirm", onclick: move |_| { result.set("Project archived.".into()); open.set(Open::None); }, "Archive project" }
            },
            DescriptionList { DescriptionItem { term: rsx! { "Project" }, "Design system refresh" } }
        }
        Dialog { open: open() == Open::Create, onclose: close, heading: HeadingLevel::H3, fc_id: create_id,
            close_label: close_form, title: rsx! { "Create a project" },
            description: rsx! { "Give this project a name. Required fields use native browser validation." },
            footer: rsx! {
                Button { variant: ButtonVariant::Cancel, r#type: ButtonType::Submit, form: "{create_form}", value: "cancel", formnovalidate: true, "Cancel" }
                Button { variant: ButtonVariant::Ok, r#type: ButtonType::Submit, form: "{create_form}", value: "create", "Create project" }
            },
            form { id: "{create_form}", class: STACK.block(),
                onsubmit: move |event| { event.prevent_default(); open.set(Open::None); },
                label { class: FIELD.block(), span { class: LABEL.block(), "Project name (required)" }
                    input { class: INPUT.block(), name: "name", required: true, maxlength: "80", autocomplete: "off" } }
                label { class: FIELD.block(), span { class: LABEL.block(), "Description (optional)" }
                    textarea { class: "{INPUT.block()} {SOFT.block()}", name: "description", rows: "3", maxlength: "240" } }
            }
        }
        Dialog { open: open() == Open::Settings, onclose: close, size: DialogSize::Lg, heading: HeadingLevel::H3, fc_id: settings_id,
            close_label: close_prefs, title: rsx! { "Conversation preferences" },
            description: rsx! { "Choose how this conversation behaves. Changes apply when you select Save preferences." },
            footer: rsx! {
                Button { variant: ButtonVariant::Cancel, r#type: ButtonType::Submit, form: "{settings_form}", value: "cancel", formnovalidate: true, "Cancel" }
                Button { variant: ButtonVariant::Ok, r#type: ButtonType::Submit, form: "{settings_form}", value: "save", "Save preferences" }
            },
            form { id: "{settings_form}", class: STACK.block(),
                onsubmit: move |event| { event.prevent_default(); open.set(Open::None); },
                fieldset { class: STACK.block(),
                    legend { "Response style" }
                    RadioScope { radios: RadioChoice { name: style_name, value: Some(style()), onchange: Some(EventHandler::new(move |v| style.set(v))) },
                        Choice { value: "concise", title: rsx! { "Concise" }, description: rsx! { "Short answers with the essential details." } }
                        Choice { value: "detailed", title: rsx! { "Detailed" }, description: rsx! { "More context, reasoning and examples." } }
                    }
                }
                Checkbox { label: rsx! { "Show source links" }, default_checked: CheckState::On }
                label { class: "{FIELD.block()} {INLINE.block()}", span { class: LABEL.block(), "Language" }
                    select { class: INPUT.block(), name: "language",
                        option { value: "en", "English" } option { value: "es", "Español" } option { value: "fr", "Français" }
                    }
                }
            }
        }
    }
}

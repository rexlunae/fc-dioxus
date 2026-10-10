//! The chat composer, from composer.css. Behaviour ports upstream's `composer.ts`.
use crate::{
    core::{
        assets,
        attrs::{
            component_attributes, composer_state_attribute, dragging_attribute,
            error_state_attribute, truncated_attribute,
        },
    },
    prelude::*,
    theme::use_stylesheets,
};
const BEM: Bem = Bem::new("fc-composer");
const BTN: Bem = Bem::new("fc-btn");
/// Roughly how many characters of a file name a chip shows before its ellipsis.
const NAME_FITS: usize = 32;
fc_modifier! { enum Frame for BEM { Open => None, Contained => "contained" } }

/// One attached file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attachment {
    /// Display name, also its remove button's name.
    pub name: String,
    /// Bytes.
    pub size: u64,
    /// Why it could not be read. A failed file is shown, never silently dropped.
    pub error: Option<String>,
}

/// "1 attachment · 2.0 KB · Shift+Enter for a new line": the composer's status line.
#[must_use]
pub fn describe(text: &str, attachments: &[Attachment], running: bool) -> String {
    let mut parts = Vec::new();
    if !attachments.is_empty() {
        let bytes: u64 = attachments.iter().map(|a| a.size).sum();
        let n = attachments.len();
        parts.push(format!("{n} attachment{}", if n == 1 { "" } else { "s" }));
        if bytes > 0 {
            parts.push(format_size(bytes));
        }
    }
    if running {
        parts.push("generating".to_owned());
    }
    if text.trim().is_empty() && attachments.is_empty() {
        parts.push("Shift+Enter for a new line".to_owned());
    }
    parts.join(" · ")
}

/// Bytes as people read them: "512 B", "2.0 KB", "1.5 MB".
#[must_use]
pub fn format_size(bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    #[allow(clippy::cast_precision_loss)]
    let value = bytes as f64;
    if value < KIB {
        format!("{bytes} B")
    } else if value < KIB * KIB {
        format!("{:.1} KB", value / KIB)
    } else {
        format!("{:.1} MB", value / KIB / KIB)
    }
}

/// Text entry, attachments, and a send control that becomes Stop while a run is in
/// flight.
///
/// A `<textarea>`, not contenteditable, so input methods, undo, and Shift+Enter work
/// natively. Enter sends unless an input method is composing. The send button changes
/// meaning, never identity, so focus does not jump. Stop does not clear the draft, and
/// the draft clears only after `onsubmit` accepts it.
#[component]
pub fn Composer(
    label: AccessibleName,
    #[props(default)] value: Option<ReadSignal<String>>,
    #[props(default)] default_value: String,
    #[props(default)] oninput: Option<EventHandler<String>>,
    onsubmit: EventHandler<(String, Vec<Attachment>)>,
    /// A run is in flight: the field is read-only and Send becomes Stop.
    #[props(default)]
    running: bool,
    #[props(default)] onstop: Option<EventHandler<()>>,
    #[props(default)] attachments: Vec<Attachment>,
    #[props(default)] onremove: Option<EventHandler<usize>>,
    /// Files dropped on the composer.
    #[props(default)]
    onfiles: Option<EventHandler<Vec<dioxus::html::FileData>>>,
    /// Controls beside the send button: attach, model, tools.
    #[props(default)]
    tools: Option<Element>,
    #[props(default)] placeholder: Option<String>,
    /// Draw the composer as its own bordered surface.
    #[props(default)]
    contained: bool,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let state = use_controllable(value, || default_value, oninput);
    let mut dragging = use_signal(|| false);
    let text = state.get();
    let empty = text.trim().is_empty() && attachments.is_empty();
    let phase = if running {
        "running"
    } else if empty {
        "idle"
    } else {
        "ready"
    };
    let mut attributes = component_attributes(
        attributes,
        ClassList::new(BEM.block())
            .with_modifier(
                BEM,
                if contained {
                    Frame::Contained
                } else {
                    Frame::Open
                },
            )
            .extend(class.as_deref()),
        &[],
    );
    attributes.push(composer_state_attribute(phase));
    attributes.extend(dragging_attribute(dragging()));
    let submit = {
        let attachments = attachments.clone();
        move || {
            let text = state.get();
            if running || (text.trim().is_empty() && attachments.is_empty()) {
                return;
            }
            onsubmit.call((text.trim().to_owned(), attachments.clone()));
            state.set(String::new());
        }
    };
    let submit_key = submit.clone();
    let submit_click = submit;
    let send_class = ClassList::new(BTN.block())
        .with(BTN.modifier_raw("primary"))
        .with(BEM.element("send"))
        .to_string();
    let status = describe(&text, &attachments, running);
    rsx! {
        if use_stylesheets() {
            document::Stylesheet { href: assets::COMPONENTS_BUTTON_CSS }
            document::Stylesheet { href: assets::COMPONENTS_COMPOSER_CSS }
        }
        div {
            ondragover: move |event| { event.prevent_default(); dragging.set(true); },
            ondragleave: move |_| dragging.set(false),
            ondrop: move |event| {
                event.prevent_default();
                dragging.set(false);
                if let Some(onfiles) = onfiles {
                    let files = event.data_transfer().files();
                    if !files.is_empty() { onfiles.call(files); }
                }
            },
            ..attributes,
            textarea { class: BEM.element("input"), rows: "2", aria_label: label.to_string(), placeholder,
                // Read-only, not disabled: a disabled field leaves the tab order.
                readonly: running,
                value: "{text}",
                oninput: move |event| state.set(event.value()),
                onkeydown: move |event| {
                    // Enter during input-method composition picks a candidate; sending
                    // there would commit half a character.
                    if event.key() != Key::Enter || event.is_composing() || event.modifiers().shift() {
                        return;
                    }
                    event.prevent_default();
                    submit_key();
                },
            }
            if !attachments.is_empty() {
                ul { class: BEM.element("attachments"),
                    for (index, attachment) in attachments.iter().enumerate() {
                        li { key: "{index}-{attachment.name}", class: BEM.element("attachment"),
                            ..error_state_attribute(attachment.error.is_some()),
                            // The name is ellipsized by the stylesheet; its full value stays in
                            // `title`, and a name long enough to be cut is declared truncated.
                            span { class: BEM.element("attachment-name"), title: "{attachment.name}",
                                ..truncated_attribute(&Clipped::chars(attachment.name.clone(), NAME_FITS)),
                                "{attachment.name}"
                            }
                            // Each remove button names its file, so four of them are not all "Remove".
                            button { r#type: "button", class: BEM.element("attachment-remove"), aria_label: "Remove {attachment.name}",
                                onclick: move |_| if let Some(onremove) = onremove { onremove.call(index); }, "×" }
                        }
                    }
                }
            }
            div { class: BEM.element("row"),
                div { class: BEM.element("tools"), {tools} }
                button { r#type: "button", class: send_class, disabled: !running && empty,
                    aria_label: if running { "Stop generating" } else { "Send message" },
                    onclick: move |_| if running { if let Some(onstop) = onstop { onstop.call(()); } } else { submit_click(); },
                    ..vec![composer_state_attribute(phase)],
                    if running { "Stop" } else { "Send" }
                }
            }
            p { class: BEM.element("status"), role: "status", "{status}" }
        }
    }
}

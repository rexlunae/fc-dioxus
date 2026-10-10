//! The chat surface: a log of message groups, from chat.css.
use crate::{
    core::{
        assets,
        attrs::{
            availability_attributes, component_attributes, full_text_title, truncated_attribute,
            FcData, Kind,
        },
    },
    prelude::*,
    theme::use_stylesheets,
};
const CHAT: Bem = Bem::new("fc-chat");
const MSG: Bem = Bem::new("fc-msg");
const ACTIONS: Bem = Bem::new("fc-msg__actions");
const PROSE: Bem = Bem::new("fc-prose");
fc_modifier! { enum ActionsVisibility for ACTIONS { Hover => None, Persistent => "persistent" } }

/// A conversation, as a `role="log"` so new messages are announced politely in order.
#[component]
pub fn Chat(
    label: AccessibleName,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let attributes = component_attributes(
        attributes,
        ClassList::new(CHAT.block()).extend(class.as_deref()),
        &["role", "aria-label"],
    );
    rsx! {
        if use_stylesheets() {
            document::Stylesheet { href: assets::COMPONENTS_CHAT_CSS }
            // Message bodies are prose, styled by the editor's document rules.
            document::Stylesheet { href: assets::COMPONENTS_EDITOR_CSS }
        }
        div { role: "log", aria_label: label.to_string(), ..attributes, {children} }
    }
}

/// Consecutive messages shown together, e.g. one turn.
#[component]
pub fn ChatGroup(children: Element) -> Element {
    rsx! { div { class: CHAT.element("group"), {children} } }
}

/// When a message was sent: a machine-readable instant and how people read it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentAt {
    /// The instant, written to `<time datetime>`.
    pub at: Timestamp,
    /// The displayed form, e.g. "09:12".
    pub display: String,
}

/// One message: author, time, body, and optional actions.
#[component]
pub fn Message(
    role: MessageRole,
    state: Lifecycle,
    author: Element,
    #[props(default)] sent: Option<SentAt>,
    /// [`MessageAction`]s.
    #[props(default)]
    actions: Option<Element>,
    /// Keep actions visible instead of revealing them on hover and focus.
    #[props(default)]
    persistent_actions: bool,
    /// The full text when the body shows only part of it; declares truncation.
    #[props(default)]
    clipped: Option<Clipped>,
    #[props(default)] record: Record,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let mut attributes = component_attributes(
        attributes,
        ClassList::new(MSG.block()).extend(class.as_deref()),
        &[],
    );
    attributes.extend(
        FcData {
            kind: Some(Kind::Message),
            id: record.id.clone(),
            seq: record.seq,
            role: Some(role),
            state: Some(state),
            ts: record.ts.clone(),
        }
        .attributes(),
    );
    // The message declares truncation; the body it shortens carries the full text.
    let title = clipped.as_ref().map(full_text_title).unwrap_or_default();
    attributes.extend(
        clipped
            .as_ref()
            .map(truncated_attribute)
            .unwrap_or_default(),
    );
    let actions_class = ClassList::new(ACTIONS.block())
        .with_modifier(
            ACTIONS,
            if persistent_actions {
                ActionsVisibility::Persistent
            } else {
                ActionsVisibility::Hover
            },
        )
        .to_string();
    let body_class = ClassList::new(&MSG.element("body"))
        .with(PROSE.block())
        .to_string();
    rsx! {
        div { ..attributes,
            div { class: MSG.element("meta"),
                span { class: MSG.element("author"), {author} }
                if let Some(sent) = sent { time { class: MSG.element("time"), datetime: "{sent.at}", "{sent.display}" } }
            }
            div { class: body_class, ..title, {children} }
            if let Some(actions) = actions {
                div { class: actions_class, role: "toolbar", aria_label: "Message actions", {actions} }
            }
        }
    }
}

/// One action on a message. Icon-only actions take `icon_label`; otherwise the visible
/// text is the name.
#[component]
pub fn MessageAction(
    #[props(default)] icon_label: Option<AccessibleName>,
    #[props(default)] pressed: Option<bool>,
    #[props(default)] availability: Availability,
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    let onclick = availability.guard(onclick);
    let state = availability_attributes(&availability);
    rsx! {
        button { r#type: "button", class: MSG.element("action"),
            aria_label: icon_label.map(|l| l.to_string()),
            aria_pressed: pressed.map(|p| p.to_string()),
            onclick: move |event| if let Some(onclick) = onclick { onclick.call(event); },
            ..state, {children}
        }
    }
}

/// One addressable unit of a message body: a text block, a code block, a tool call.
#[component]
pub fn MessagePart(children: Element) -> Element {
    rsx! { div { class: MSG.element("part"), {children} } }
}

//! A rich text editor from editor.css, with `Prose` for rendered rich text.
//!
//! The markup is Rust; the behaviour is upstream's own `editor.ts`, shipped unchanged as
//! JavaScript and bound through [`DomBridge::bind_editor`]. Editing is selection work,
//! which upstream keeps out of its Rust models on purpose: where the caret is, what the
//! IME is composing, and how to wrap a selection without collapsing it are questions
//! only the browser can answer. Without a script host the editor stays plain editable
//! text with an inert toolbar.
use crate::{
    core::{
        assets,
        attrs::{
            component_attributes, editor_block_attribute, editor_mark_attribute,
            named_state_attribute,
        },
        platform::{DomBridge, EditorEvent},
    },
    models::rich_text::Mark,
    prelude::*,
    theme::use_stylesheets,
};
const BEM: Bem = Bem::new("fc-editor");
const PROSE: Bem = Bem::new("fc-prose");
const TOOLBAR: Bem = Bem::new("fc-toolbar");
const HIDDEN: Bem = Bem::new("fc-u-visually-hidden");

/// Whether the document is being saved.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum EditorState {
    /// Editing.
    #[default]
    Idle,
    /// A save is in flight: the content dims but stays editable.
    Saving,
    /// The last save failed.
    Error,
}

/// A block a toolbar button toggles on the paragraph holding the selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockTag {
    /// A top-level heading.
    H1,
    /// A section heading.
    H2,
    /// A subsection heading.
    H3,
    /// Preformatted code.
    Pre,
    /// A quotation.
    Blockquote,
}
impl BlockTag {
    const fn command(self) -> &'static str {
        match self {
            Self::H1 => "h1",
            Self::H2 => "h2",
            Self::H3 => "h3",
            Self::Pre => "pre",
            Self::Blockquote => "blockquote",
        }
    }
    const fn glyph(self) -> (&'static str, &'static str) {
        match self {
            Self::H1 => ("H1", "Heading 1"),
            Self::H2 => ("H2", "Heading 2"),
            Self::H3 => ("H3", "Heading 3"),
            Self::Pre => ("{ }", "Code block"),
            Self::Blockquote => ("\u{201c}", "Quote"),
        }
    }
}

const fn mark_command(mark: Mark) -> (&'static str, &'static str, &'static str) {
    match mark {
        Mark::Bold => ("bold", "B", "Bold"),
        Mark::Italic => ("italic", "I", "Italic"),
        Mark::Underline => ("underline", "U", "Underline"),
        Mark::Strike => ("strike", "S", "Strikethrough"),
        Mark::Code => ("code", "</>", "Code"),
        Mark::Link => ("link", "\u{1f517}", "Link"),
    }
}

/// The marks a default toolbar offers, in upstream's order.
pub const DEFAULT_MARKS: [Mark; 5] = [
    Mark::Bold,
    Mark::Italic,
    Mark::Underline,
    Mark::Strike,
    Mark::Code,
];

/// A rich text editor: a formatting toolbar over an editable document. Toolbar buttons
/// report `aria-pressed` across the whole selection, so a half-bold selection reads as
/// not bold and a click bolds the rest. The document is reported as HTML through
/// `onchange`.
#[component]
pub fn Editor(
    /// Names the document.
    label: AccessibleName,
    /// The starting document, read once. It becomes `innerHTML`, so it must come from
    /// the application's own state, never from untrusted input.
    #[props(default)]
    initial: String,
    /// Shown while the document is empty.
    #[props(default)]
    placeholder: Option<String>,
    #[props(default)] state: EditorState,
    #[props(default)] readonly: bool,
    /// The mark buttons, in order.
    #[props(default = DEFAULT_MARKS.to_vec())]
    marks: Vec<Mark>,
    /// The block buttons, after a divider.
    #[props(default)]
    blocks: Vec<BlockTag>,
    #[props(default)] onchange: Option<EventHandler<String>>,
    /// The marks covering the whole selection, as the toolbar shows them.
    #[props(default)]
    onmarks: Option<EventHandler<Vec<Mark>>>,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let id = use_id();
    // Read once: after mount the document belongs to the browser and the binding, and
    // re-rendering `innerHTML` would throw away the caret and the user's typing.
    let initial = use_hook(|| initial.clone());
    let mut attributes = component_attributes(
        attributes,
        ClassList::new(BEM.block()).extend(class.as_deref()),
        &["id"],
    );
    attributes.push(Attribute::new("id", id.to_string(), None, false));
    match state {
        EditorState::Idle => {}
        EditorState::Saving => attributes.push(named_state_attribute("saving")),
        EditorState::Error => attributes.push(named_state_attribute("error")),
    }
    let content_class = ClassList::new(&BEM.element("content"))
        .with(PROSE.block())
        .to_string();
    let icon_class = ClassList::new(&TOOLBAR.element("item"))
        .with(TOOLBAR.element("item--icon"))
        .to_string();

    rsx! {
        if use_stylesheets() {
            document::Stylesheet { href: assets::COMPONENTS_TOOLBAR_CSS }
            document::Stylesheet { href: assets::COMPONENTS_EDITOR_CSS }
        }
        div {
            onmounted: move |_| {
                let id = id.clone();
                async move {
                    let Ok(mut session) = DomBridge.bind_editor(&id).await else { return };
                    while let Some(event) = session.next().await {
                        match event {
                            EditorEvent::Change(html) => if let Some(onchange) = onchange { onchange.call(html) },
                            EditorEvent::Marks(active) => if let Some(onmarks) = onmarks { onmarks.call(active) },
                        }
                    }
                }
            },
            ..attributes,
            // The binding owns `aria-pressed` and the roving tabindex from here on;
            // neither is rendered dynamically, so the two never contend.
            div { class: TOOLBAR.block(), role: "toolbar", aria_label: "Formatting",
                for mark in marks.iter().copied() {
                    {
                        let (command, glyph, name) = mark_command(mark);
                        rsx! {
                            button { key: "{command}", r#type: "button", class: TOOLBAR.element("item"), aria_pressed: "false",
                                disabled: readonly, ..crate::spread![editor_mark_attribute(command)],
                                span { aria_hidden: "true", "{glyph}" }
                                span { class: HIDDEN.block(), "{name}" }
                            }
                        }
                    }
                }
                if !blocks.is_empty() {
                    span { class: TOOLBAR.element("divider"), role: "separator" }
                }
                for block in blocks.iter().copied() {
                    {
                        let (glyph, name) = block.glyph();
                        rsx! {
                            // The name is spoken from the hidden span, so "H2" on screen and
                            // "Heading 2" in speech do not count as a label mismatch.
                            button { key: "{block.command()}", r#type: "button", class: "{icon_class}",
                                disabled: readonly, ..crate::spread![editor_block_attribute(block.command())],
                                span { aria_hidden: "true", "{glyph}" }
                                span { class: HIDDEN.block(), "{name}" }
                            }
                        }
                    }
                }
            }
            div { class: "{content_class}", role: "textbox", aria_multiline: "true", aria_label: "{label}",
                contenteditable: if readonly { "false" } else { "true" },
                aria_readonly: if readonly { Some("true") } else { None },
                "data-placeholder": placeholder,
                dangerous_inner_html: "{initial}",
            }
        }
    }
}

/// Rendered rich text: headings, lists, quotes, code, and links in upstream's prose
/// styles.
#[component]
pub fn Prose(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let attributes = component_attributes(attributes, ClassList::new(PROSE.block()), &[]);
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_EDITOR_CSS } }
        div { ..attributes, {children} }
    }
}

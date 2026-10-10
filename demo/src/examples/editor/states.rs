use fc_dioxus::models::rich_text::Mark;
use fc_dioxus::prelude::*;

const DOCUMENT: &str = "<p>Some <strong>bold</strong> and <code>code</code>.</p>";

// example:start
/// A live editor that reports its HTML and the marks across the selection; saving,
/// failed, read-only, and empty editors; and the same styles as rendered prose.
#[component]
pub fn States() -> Element {
    const STACK: Bem = Bem::new("fc-stack");
    const PREWRAP: Bem = Bem::new("fc-u-prewrap");
    let mut html = use_signal(|| DOCUMENT.to_owned());
    let mut marks = use_signal(Vec::<Mark>::new);
    let mut drafting = use_signal(|| true);
    let active = marks()
        .iter()
        .map(|mark| format!("{mark:?}"))
        .collect::<Vec<_>>()
        .join(", ");
    rsx! {
        div { class: STACK.block(),
            Editor {
                label: name!("Document"),
                initial: TrustedHtml::assume_trusted(DOCUMENT),
                blocks: vec![BlockTag::H2, BlockTag::Blockquote, BlockTag::Pre],
                onchange: move |next| html.set(next),
                onmarks: move |next| marks.set(next),
            }
            p { role: "status", "Marks: {active}" }
            pre { class: PREWRAP.block(), "{html}" }
            Editor { label: name!("Saving document"), initial: TrustedHtml::assume_trusted(DOCUMENT), state: EditorState::Saving }
            Editor { label: name!("Unsaved document"), initial: TrustedHtml::assume_trusted(DOCUMENT), state: EditorState::Error }
            Editor { label: name!("Published document"), initial: TrustedHtml::assume_trusted(DOCUMENT), readonly: true }
            // Removing an editor releases its binding: no listener outlives it.
            Button { variant: ButtonVariant::Secondary, onclick: move |_| drafting.toggle(),
                if drafting() { "Discard the new document" } else { "Start a new document" }
            }
            if drafting() {
                Editor { label: name!("New document"), placeholder: "Start writing…".to_owned() }
            }
            Prose {
                h2 { "Rendered prose" }
                p { "Text with " strong { "strong" } ", " em { "emphasis" } ", " code { "code" } ", a " a { href: "#editor", "link" } ", " del { "removed" } " and " ins { "inserted" } " text." }
                ul { li { "A list item" } li { "Another" } }
                blockquote { p { "A quotation." } }
                pre { code { "fn main() {{}}" } }
                hr {}
            }
        }
    }
}

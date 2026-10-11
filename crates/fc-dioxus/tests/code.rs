//! Code blocks highlight through upstream's fc-highlight and never alter the text.
#![forbid(unsafe_code)]

use fc_dioxus::prelude::*;

#[test]
fn highlighted_rust_draws_role_spans_that_rebuild_the_source() {
    let source = "fn main() { println!(\"hi\", 42); } // done";
    let spans = match CodeSource::highlighted(source, "rs") {
        CodeSource::Spans(spans) => spans,
        CodeSource::Plain(_) => Vec::new(),
    };
    assert!(!spans.is_empty(), "rust is a supported language");
    assert_eq!(spans.iter().map(|s| s.text.as_str()).collect::<String>(), source);
    let html = dioxus_ssr::render_element(rsx! {
        FutureCss { stylesheets: Stylesheets::External,
            CodeBlock { source: CodeSource::Spans(spans), lang: "rust" }
        }
    });
    for role in ["keyword", "function", "string", "number", "comment"] {
        assert!(html.contains(&format!("fc-code__tok--{role}")), "{role}: {html}");
    }
}

#[test]
fn an_unknown_language_is_plain_text() {
    assert!(!CodeSource::supports("cobol"));
    assert_eq!(
        CodeSource::highlighted("DISPLAY 'HELLO'.", "cobol"),
        CodeSource::Plain("DISPLAY 'HELLO'.".to_owned())
    );
}

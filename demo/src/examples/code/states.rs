use fc_dioxus::prelude::*;

// example:start
/// Highlighted Rust with line numbers, a scrolling CSS sample, every syntax role, and a
/// language no highlighter knows. `CodeSource::highlighted` tokenizes with upstream's
/// highlighter; spans from any other highlighter draw the same way.
#[component]
pub fn States() -> Element {
    const STACK: Bem = Bem::new("fc-stack");
    let rust = CodeSource::highlighted(
        "// Greets someone by name.\nfn greet(name: &str) -> String {\n    format!(\"Hello, {name}! {}\", 42)\n}",
        "rust",
    );
    let every: Vec<SyntaxSpan> = SyntaxRole::ALL
        .iter()
        .map(|role| SyntaxSpan::new(*role, format!("{role:?} ")))
        .collect();
    rsx! {
        div { class: STACK.block(),
            CodeBlock { source: rust, lang: "rust", layout: CodeLayout::Numbered }
            CodeBlock { source: CodeSource::highlighted(".fc-code { inline-size: 100%; overflow-x: auto; scrollbar-gutter: stable; }", "css"),
                lang: "css", layout: CodeLayout::Scroll(name!("CSS sample")) }
            CodeBlock { source: CodeSource::Spans(every), lang: "roles" }
            CodeBlock { source: CodeSource::Plain("DISPLAY 'HELLO'.".into()), lang: "cobol", unsupported: true }
        }
    }
}

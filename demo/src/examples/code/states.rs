use fc_dioxus::prelude::*;

// example:start
/// Highlighted Rust with line numbers, a scrolling CSS sample, every syntax role, and a
/// language no highlighter knows. The spans come from your highlighter of choice; the
/// block only draws them.
#[component]
pub fn States() -> Element {
    const STACK: Bem = Bem::new("fc-stack");
    use SyntaxRole::{
        Comment, Fg, Function, Keyword, Number, Operator, Punctuation, String as Str, Type,
    };
    let rust = vec![
        SyntaxSpan::new(Comment, "// Greets someone by name.\n"),
        SyntaxSpan::new(Keyword, "fn"),
        SyntaxSpan::new(Fg, " "),
        SyntaxSpan::new(Function, "greet"),
        SyntaxSpan::new(Punctuation, "("),
        SyntaxSpan::new(Fg, "name"),
        SyntaxSpan::new(Punctuation, ": "),
        SyntaxSpan::new(Operator, "&"),
        SyntaxSpan::new(Type, "str"),
        SyntaxSpan::new(Punctuation, ") -> "),
        SyntaxSpan::new(Type, "String"),
        SyntaxSpan::new(Punctuation, " {\n    "),
        SyntaxSpan::new(Function, "format!"),
        SyntaxSpan::new(Punctuation, "("),
        SyntaxSpan::new(Str, "\"Hello, {name}! {}\""),
        SyntaxSpan::new(Punctuation, ", "),
        SyntaxSpan::new(Number, "42"),
        SyntaxSpan::new(Punctuation, ")\n}"),
    ];
    let every: Vec<SyntaxSpan> = SyntaxRole::ALL
        .iter()
        .map(|role| SyntaxSpan::new(*role, format!("{role:?} ")))
        .collect();
    rsx! {
        div { class: STACK.block(),
            CodeBlock { source: CodeSource::Spans(rust), lang: "rust", layout: CodeLayout::Numbered }
            CodeBlock { source: CodeSource::Plain(".fc-code { inline-size: 100%; overflow-x: auto; scrollbar-gutter: stable; }".into()),
                lang: "css", layout: CodeLayout::Scroll(name!("CSS sample")) }
            CodeBlock { source: CodeSource::Spans(every), lang: "roles" }
            CodeBlock { source: CodeSource::Plain("DISPLAY 'HELLO'.".into()), lang: "cobol", unsupported: true }
        }
    }
}

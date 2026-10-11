//! Source code for reading, from code.css.
//!
//! A [`CodeBlock`] draws spans it is given. [`CodeSource::highlighted`] makes them with
//! upstream's `fc-highlight`, the Rust twin of `highlight.ts`, so a block tokenizes the
//! same way on every renderer as it does in the browser; any other highlighter's spans
//! work too. Every span sits in one `<code>`, so its text is byte-identical to the
//! source.
use crate::{
    core::{
        assets,
        attrs::{component_attributes, lang_attribute},
    },
    prelude::*,
    theme::use_stylesheets,
};
const BEM: Bem = Bem::new("fc-code");
const TOK: Bem = Bem::new("fc-code__tok");
fc_modifier! { enum Layout for BEM { Wrap => None, Numbered => "numbered", Scroll => "scroll" } }
fc_modifier! { enum Support for BEM { Highlighted => None, Unsupported => "unsupported" } }

/// A syntax role; each has a `--fc-code-<role>` color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SyntaxRole {
    /// Plain text.
    Fg,
    /// A comment.
    Comment,
    /// A keyword.
    Keyword,
    /// A string literal.
    String,
    /// A number.
    Number,
    /// A function name.
    Function,
    /// A type name.
    Type,
    /// An operator.
    Operator,
    /// Punctuation.
    Punctuation,
    /// A property or field.
    Property,
    /// A constant such as `true`.
    Constant,
    /// A markup tag.
    Tag,
    /// A markup attribute.
    Attribute,
    /// A regular expression.
    Regexp,
    /// A heading in prose formats.
    Heading,
    /// A link.
    Link,
    /// Something invalid.
    Invalid,
}
impl SyntaxRole {
    /// Every role.
    pub const ALL: [Self; 17] = [
        Self::Fg,
        Self::Comment,
        Self::Keyword,
        Self::String,
        Self::Number,
        Self::Function,
        Self::Type,
        Self::Operator,
        Self::Punctuation,
        Self::Property,
        Self::Constant,
        Self::Tag,
        Self::Attribute,
        Self::Regexp,
        Self::Heading,
        Self::Link,
        Self::Invalid,
    ];
    const fn suffix(self) -> &'static str {
        match self {
            Self::Fg => "fg",
            Self::Comment => "comment",
            Self::Keyword => "keyword",
            Self::String => "string",
            Self::Number => "number",
            Self::Function => "function",
            Self::Type => "type",
            Self::Operator => "operator",
            Self::Punctuation => "punctuation",
            Self::Property => "property",
            Self::Constant => "constant",
            Self::Tag => "tag",
            Self::Attribute => "attribute",
            Self::Regexp => "regexp",
            Self::Heading => "heading",
            Self::Link => "link",
            Self::Invalid => "invalid",
        }
    }
}

/// A run of text sharing a role. Spans are contiguous and do not overlap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxSpan {
    /// Its role.
    pub role: SyntaxRole,
    /// Its text.
    pub text: String,
}
impl SyntaxSpan {
    /// A span.
    #[must_use]
    pub fn new(role: SyntaxRole, text: impl Into<String>) -> Self {
        Self {
            role,
            text: text.into(),
        }
    }
}

/// What a code block shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodeSource {
    /// Unhighlighted text.
    Plain(String),
    /// Highlighted spans whose texts concatenate to the source.
    Spans(Vec<SyntaxSpan>),
}
impl CodeSource {
    /// `source` tokenized by upstream's highlighter, or plain text when it knows no
    /// language by this name or alias (see [`CodeSource::supports`]).
    #[must_use]
    pub fn highlighted(source: &str, lang: &str) -> Self {
        if !Self::supports(lang) {
            return Self::Plain(source.to_owned());
        }
        Self::Spans(
            fc_highlight::highlight(source, lang)
                .into_iter()
                .map(|span| SyntaxSpan::new(role(span.role), span.text))
                .collect(),
        )
    }

    /// Whether upstream's highlighter knows `lang`, as a name ("rust") or an alias
    /// ("rs"). A block of any other language sets `unsupported`.
    #[must_use]
    pub fn supports(lang: &str) -> bool {
        fc_highlight::Language::resolve(lang).is_some()
    }

    fn text(&self) -> String {
        match self {
            Self::Plain(text) => text.clone(),
            Self::Spans(spans) => spans.iter().map(|s| s.text.as_str()).collect(),
        }
    }
}

/// The upstream role's twin. Both enums mirror `highlight.ts`'s `SyntaxRole`.
const fn role(role: fc_highlight::Role) -> SyntaxRole {
    use fc_highlight::Role as R;
    match role {
        R::Fg => SyntaxRole::Fg,
        R::Comment => SyntaxRole::Comment,
        R::Keyword => SyntaxRole::Keyword,
        R::String => SyntaxRole::String,
        R::Number => SyntaxRole::Number,
        R::Function => SyntaxRole::Function,
        R::Type => SyntaxRole::Type,
        R::Operator => SyntaxRole::Operator,
        R::Punctuation => SyntaxRole::Punctuation,
        R::Property => SyntaxRole::Property,
        R::Constant => SyntaxRole::Constant,
        R::Tag => SyntaxRole::Tag,
        R::Attribute => SyntaxRole::Attribute,
        R::Regexp => SyntaxRole::Regexp,
        R::Heading => SyntaxRole::Heading,
        R::Link => SyntaxRole::Link,
        R::Invalid => SyntaxRole::Invalid,
    }
}

/// How a code block treats long lines.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum CodeLayout {
    /// Wrap long lines.
    #[default]
    Wrap,
    /// Wrap, with a line-number gutter (one number per logical line).
    Numbered,
    /// Keep lines whole and scroll sideways, in a focusable region named by this.
    Scroll(AccessibleName),
}

/// A read-only block of source code.
#[component]
pub fn CodeBlock(
    source: CodeSource,
    /// The language, as `data-fc-lang`, e.g. "rust".
    #[props(default)]
    lang: Option<String>,
    #[props(default)] layout: CodeLayout,
    /// No highlighter knows this language: show its name in a bar above the text.
    #[props(default)]
    unsupported: bool,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let shape = match layout {
        CodeLayout::Wrap => Layout::Wrap,
        CodeLayout::Numbered => Layout::Numbered,
        CodeLayout::Scroll(_) => Layout::Scroll,
    };
    let mut attributes = component_attributes(
        attributes,
        ClassList::new(BEM.block())
            .with_modifier(BEM, shape)
            .with_modifier(
                BEM,
                if unsupported {
                    Support::Unsupported
                } else {
                    Support::Highlighted
                },
            )
            .extend(class.as_deref()),
        &["dir", "role", "aria-label", "tabindex"],
    );
    if let Some(lang) = &lang {
        attributes.push(lang_attribute(lang));
    }
    if let CodeLayout::Scroll(label) = &layout {
        attributes.push(Attribute::new("role", "region", None, false));
        attributes.push(Attribute::new("aria-label", label.to_string(), None, false));
        attributes.push(Attribute::new("tabindex", "0", None, false));
    }
    let lines = source.text().split('\n').count();
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_CODE_CSS } }
        // Code reads left to right in every document.
        div { dir: "ltr", ..attributes,
            if unsupported {
                div { class: BEM.element("bar"), span { class: BEM.element("lang"), "{lang.clone().unwrap_or_default()}" } }
            }
            if layout == CodeLayout::Numbered {
                div { class: BEM.element("gutter"), aria_hidden: "true",
                    for number in 1..=lines { span { key: "{number}", "{number}" } }
                }
            }
            code { class: BEM.element("source"), tabindex: "0",
                match &source {
                    CodeSource::Plain(text) => rsx! { "{text}" },
                    CodeSource::Spans(spans) => rsx! {
                        for (index, span) in spans.iter().enumerate() {
                            span { key: "{index}", class: TOK.modifier_raw(span.role.suffix()), "{span.text}" }
                        }
                    },
                }
            }
        }
    }
}

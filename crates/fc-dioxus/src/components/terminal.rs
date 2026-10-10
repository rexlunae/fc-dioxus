//! A terminal transcript from terminal.css: styled output lines, a prompt, a status
//! bar with the exit code, and a cursor.
//!
//! It parses no escape sequences. Output arrives as styled runs; turning raw ANSI into
//! runs is `fc-ansi`, a model crate requested upstream (see COMPONENTS.md, "Waiting on
//! upstream"), so the parser lands once, with a TypeScript twin, rather than here.
use crate::{
    core::{
        assets,
        attrs::{component_attributes, terminal_state_attribute},
    },
    prelude::*,
    theme::use_stylesheets,
};
use dioxus::html::geometry::PixelsVector2D;
use std::rc::Rc;
const BEM: Bem = Bem::new("fc-terminal");

/// The sixteen ANSI colours, drawn from themed tokens rather than fixed values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum AnsiColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}
impl AnsiColor {
    /// Every colour, in SGR order (30 to 37, then 90 to 97).
    pub const ALL: [Self; 16] = [
        Self::Black,
        Self::Red,
        Self::Green,
        Self::Yellow,
        Self::Blue,
        Self::Magenta,
        Self::Cyan,
        Self::White,
        Self::BrightBlack,
        Self::BrightRed,
        Self::BrightGreen,
        Self::BrightYellow,
        Self::BrightBlue,
        Self::BrightMagenta,
        Self::BrightCyan,
        Self::BrightWhite,
    ];
    /// The colour's class, a block of its own in terminal.css.
    #[must_use]
    pub const fn bem(self) -> Bem {
        match self {
            Self::Black => Bem::new("fc-term-ansi-black"),
            Self::Red => Bem::new("fc-term-ansi-red"),
            Self::Green => Bem::new("fc-term-ansi-green"),
            Self::Yellow => Bem::new("fc-term-ansi-yellow"),
            Self::Blue => Bem::new("fc-term-ansi-blue"),
            Self::Magenta => Bem::new("fc-term-ansi-magenta"),
            Self::Cyan => Bem::new("fc-term-ansi-cyan"),
            Self::White => Bem::new("fc-term-ansi-white"),
            Self::BrightBlack => Bem::new("fc-term-ansi-bright-black"),
            Self::BrightRed => Bem::new("fc-term-ansi-bright-red"),
            Self::BrightGreen => Bem::new("fc-term-ansi-bright-green"),
            Self::BrightYellow => Bem::new("fc-term-ansi-bright-yellow"),
            Self::BrightBlue => Bem::new("fc-term-ansi-bright-blue"),
            Self::BrightMagenta => Bem::new("fc-term-ansi-bright-magenta"),
            Self::BrightCyan => Bem::new("fc-term-ansi-bright-cyan"),
            Self::BrightWhite => Bem::new("fc-term-ansi-bright-white"),
        }
    }
}

/// A text attribute a run can carry, by its SGR code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sgr {
    /// SGR 1.
    Bold,
    /// SGR 2.
    Dim,
    /// SGR 3.
    Italic,
    /// SGR 4.
    Underline,
    /// SGR 7: swaps foreground and background.
    Inverse,
    /// SGR 9.
    Strike,
}
impl Sgr {
    /// Every attribute.
    pub const ALL: [Self; 6] = [
        Self::Bold,
        Self::Dim,
        Self::Italic,
        Self::Underline,
        Self::Inverse,
        Self::Strike,
    ];
    const fn bit(self) -> u8 {
        match self {
            Self::Bold => 1,
            Self::Dim => 2,
            Self::Italic => 4,
            Self::Underline => 8,
            Self::Inverse => 16,
            Self::Strike => 32,
        }
    }
    const fn element(self) -> &'static str {
        match self {
            Self::Bold => "bold",
            Self::Dim => "dim",
            Self::Italic => "italic",
            Self::Underline => "underline",
            Self::Inverse => "inverse",
            Self::Strike => "strike",
        }
    }
}

/// A set of [`Sgr`] attributes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct SgrSet(u8);
impl SgrSet {
    /// Whether the set holds `attribute`.
    #[must_use]
    pub const fn contains(self, attribute: Sgr) -> bool {
        self.0 & attribute.bit() != 0
    }
    /// The set with `attribute` added.
    #[must_use]
    pub const fn with(self, attribute: Sgr) -> Self {
        Self(self.0 | attribute.bit())
    }
    /// Whether the set is empty.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

/// One run of identically styled text.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TermRun {
    /// The text.
    pub text: String,
    /// Foreground colour; the terminal's own when `None`.
    pub fg: Option<AnsiColor>,
    /// Bold, dim, italic, and the rest.
    pub attributes: SgrSet,
}
impl TermRun {
    /// Unstyled text.
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Self::default()
        }
    }
    /// With a foreground colour.
    #[must_use]
    pub const fn fg(mut self, color: AnsiColor) -> Self {
        self.fg = Some(color);
        self
    }
    /// With a text attribute.
    #[must_use]
    pub const fn with(mut self, attribute: Sgr) -> Self {
        self.attributes = self.attributes.with(attribute);
        self
    }
    fn styled(&self) -> bool {
        self.fg.is_some() || !self.attributes.is_empty()
    }
    fn classes(&self) -> String {
        Sgr::ALL
            .into_iter()
            .filter(|a| self.attributes.contains(*a))
            .fold(ClassList::default(), |list, a| {
                list.with(BEM.element(a.element()))
            })
            .with(self.fg.map_or("", |c| c.bem().block()))
            .to_string()
    }
}
impl<T: Into<String>> From<T> for TermRun {
    fn from(text: T) -> Self {
        Self::new(text)
    }
}

/// One line of output.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TermLine(pub Vec<TermRun>);
impl TermLine {
    /// A line of unstyled text.
    #[must_use]
    pub fn plain(text: impl Into<String>) -> Self {
        Self(vec![TermRun::new(text)])
    }
}
impl From<Vec<TermRun>> for TermLine {
    fn from(runs: Vec<TermRun>) -> Self {
        Self(runs)
    }
}

/// Whether the session behind the terminal is live.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TermState {
    /// Accepting input.
    #[default]
    Connected,
    /// The session ended or dropped: input is disabled and dimmed.
    Disconnected,
}

/// How the cursor is drawn at the end of the output.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CursorStyle {
    /// No cursor.
    #[default]
    None,
    /// A solid block.
    Steady,
    /// A blinking block; steady under reduced motion, which the CSS handles.
    Blink,
}

/// A terminal transcript. The output is a polite `log`, so new lines are announced
/// without interrupting, and it stays pinned to the newest line unless the reader has
/// scrolled up. With `onsubmit`, an input row sends commands; Up and Down recall
/// earlier ones.
#[component]
pub fn Terminal(
    output: ReadSignal<Vec<TermLine>>,
    /// Names the output region; "Terminal output" when omitted.
    #[props(default)]
    label: Option<AccessibleName>,
    #[props(default)] state: TermState,
    /// The prompt drawn before the input.
    #[props(default = "$".to_owned())]
    prompt: String,
    /// The shell or session name at the start of the status bar.
    #[props(default)]
    session: Option<String>,
    /// The last command's exit code; zero is success.
    #[props(default)]
    exit: Option<i32>,
    #[props(default)] cursor: CursorStyle,
    #[props(default)] onsubmit: Option<EventHandler<String>>,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let mut draft = use_signal(String::new);
    let mut history = use_signal(Vec::<String>::new);
    let mut recall = use_signal(|| None::<usize>);
    let mut pinned = use_signal(|| true);
    let mut node = use_signal(|| None::<Rc<MountedData>>);
    let disconnected = state == TermState::Disconnected;
    let label = label.map_or_else(|| "Terminal output".to_owned(), |l| l.to_string());

    // Follow new output only while the reader is at the end; scrolling up to read
    // must not be undone by the next line.
    use_effect(move || {
        let _ = output.read().len();
        if *pinned.peek() {
            if let Some(node) = node.peek().clone() {
                spawn(async move {
                    // Measured, not a huge constant: browsers do not all clamp an
                    // out-of-range offset to the end.
                    if let Ok(size) = node.get_scroll_size().await {
                        let _ = node
                            .scroll(
                                PixelsVector2D::new(0.0, size.height),
                                ScrollBehavior::Instant,
                            )
                            .await;
                    }
                });
            }
        }
    });

    let mut attributes = component_attributes(
        attributes,
        ClassList::new(BEM.block()).extend(class.as_deref()),
        &[],
    );
    attributes.push(terminal_state_attribute(if disconnected {
        "disconnected"
    } else {
        "connected"
    }));
    let lines = output.read().clone();
    let last = lines.len().saturating_sub(1);
    let show_cursor = cursor != CursorStyle::None && !disconnected;
    let cursor_class = ClassList::new(&BEM.element("cursor"))
        .with_if(cursor == CursorStyle::Blink, BEM.element("cursor--blink"))
        .to_string();

    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_TERMINAL_CSS } }
        // Terminal output is left-to-right in every locale; columns line up by position.
        div { dir: "ltr", ..attributes,
            div { class: BEM.element("output"), tabindex: "0", role: "log", aria_live: "polite", aria_label: "{label}",
                onmounted: move |event: MountedEvent| node.set(Some(event.data())),
                onscroll: move |event: ScrollEvent| {
                    let end = f64::from(event.scroll_height() - event.client_height());
                    // A pixel of slack: fractional scroll positions never quite reach the end.
                    pinned.set(event.scroll_top() >= end - 1.0);
                },
                for (index, line) in lines.into_iter().enumerate() {
                    div { key: "{index}", class: BEM.element("line"),
                        for run in line.0 {
                            if run.styled() {
                                span { class: run.classes(), "{run.text}" }
                            } else {
                                "{run.text}"
                            }
                        }
                        if show_cursor && index == last {
                            span { class: "{cursor_class}", aria_hidden: "true", "\u{a0}" }
                        }
                    }
                }
            }
            if let Some(onsubmit) = onsubmit {
                div { class: BEM.element("input-row"),
                    span { class: BEM.element("prompt"), aria_hidden: "true", "{prompt}" }
                    input { class: BEM.element("input"), r#type: "text", aria_label: "Terminal input",
                        autocomplete: "off", spellcheck: "false", disabled: disconnected, value: "{draft}",
                        oninput: move |event| { draft.set(event.value()); recall.set(None); },
                        onkeydown: move |event: KeyboardEvent| {
                            let key = event.key();
                            if key == Key::Enter {
                                event.prevent_default();
                                let command = draft();
                                if !command.trim().is_empty() {
                                    history.write().push(command.clone());
                                }
                                draft.set(String::new());
                                recall.set(None);
                                pinned.set(true);
                                onsubmit.call(command);
                            } else if key == Key::ArrowUp || key == Key::ArrowDown {
                                let past = history.peek().len();
                                if past == 0 { return; }
                                event.prevent_default();
                                let next = match (recall(), key == Key::ArrowUp) {
                                    (None, true) => Some(past - 1),
                                    (None, false) => None,
                                    (Some(i), true) => Some(i.saturating_sub(1)),
                                    (Some(i), false) => (i + 1 < past).then_some(i + 1),
                                };
                                recall.set(next);
                                draft.set(next.and_then(|i| history.peek().get(i).cloned()).unwrap_or_default());
                            }
                        },
                    }
                }
            }
            if session.is_some() || exit.is_some() {
                div { class: BEM.element("status"),
                    span { "{session.clone().unwrap_or_default()}" }
                    span { class: BEM.element("status-end"),
                        if let Some(code) = exit {
                            span { class: BEM.element("exit"), ..vec![terminal_state_attribute(if code == 0 { "complete" } else { "error" })],
                                "exit {code}"
                            }
                        }
                    }
                }
            }
        }
    }
}

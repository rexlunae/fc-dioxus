use fc_dioxus::core::platform::DomBridge;
use fc_dioxus::prelude::*;

fn transcript() -> Vec<TermLine> {
    use AnsiColor::{Blue, Cyan, Green, Magenta, Red, Yellow};
    vec![
        vec![TermRun::new("$").with(Sgr::Bold), " npm test".into()].into(),
        vec![
            TermRun::new("pass").fg(Green),
            " 18 ".into(),
            TermRun::new("dim").with(Sgr::Dim),
            " ".into(),
            TermRun::new("italic").with(Sgr::Italic),
            " ".into(),
            TermRun::new("underline").with(Sgr::Underline),
            " ".into(),
            TermRun::new("strike").with(Sgr::Strike),
            " ".into(),
            TermRun::new("inverse").with(Sgr::Inverse),
        ]
        .into(),
        vec![
            TermRun::new("fail").fg(Red),
            " 1 test failed ".into(),
            TermRun::new("warn").fg(Yellow),
            " ".into(),
            TermRun::new("info").fg(Blue),
            " ".into(),
            TermRun::new("mag").fg(Magenta),
            " ".into(),
            TermRun::new("cyan").fg(Cyan),
        ]
        .into(),
        palette(),
    ]
}

/// Each ANSI colour drawn as its own SGR code, normal then bright: text a screen reader
/// can read, unlike a row of block glyphs.
fn palette() -> TermLine {
    AnsiColor::ALL
        .iter()
        .zip((30..38).chain(90..98))
        .map(|(color, code)| TermRun::new(format!("{code} ")).fg(*color))
        .collect::<Vec<_>>()
        .into()
}

// example:start
/// A live session under an editor pane that runs `help`, `colors`, `watch`, and `clear`, with a
/// blinking cursor and history on Up and Down; and a dropped session whose last command
/// failed. A terminal fills its container's height, so the live one sits in a split.
#[component]
pub fn States() -> Element {
    const STACK: Bem = Bem::new("fc-stack");
    let mut output = use_signal(transcript);
    let mut exit = use_signal(|| Some(0));
    rsx! {
        div { class: STACK.block(),
            Split { label: name!("Editor and terminal"), stacked: true, block_size: "16rem", default_value: 25.0, min: 10.0, max: 80.0,
                primary: rsx! { "Editor" },
                secondary: rsx! {
                    Terminal {
                        output,
                        session: "bash".to_owned(),
                        exit: exit(),
                        cursor: CursorStyle::Blink,
                        onsubmit: move |command: String| {
                            // A copy of the handle for output that arrives after this borrow ends.
                    let mut feed = output;
                    let mut lines = output.write();
                            lines.push(vec![TermRun::new("$").with(Sgr::Bold), format!(" {command}").into()].into());
                            match command.trim() {
                                "" => {}
                                "help" => {
                                    lines.push(TermLine::plain("commands: help, colors, watch, clear"));
                                    exit.set(Some(0));
                                }
                                "colors" => {
                                    lines.push(palette());
                                    exit.set(Some(0));
                                }
                                "watch" => {
                            // Output that keeps arriving after the command, like a build:
                            // scroll up and it stays where you are.
                            spawn(async move {
                                for n in 1..=8 {
                                    if DomBridge.delay(150).await.is_err() {
                                        break;
                                    }
                                    feed.write().push(vec![TermRun::new("watch").fg(AnsiColor::Cyan), format!(" rebuilt in {}ms", 40 + n * 3).into()].into());
                                }
                            });
                            exit.set(None);
                        }
                        "clear" => {
                                    lines.clear();
                                    exit.set(Some(0));
                                }
                                other => {
                                    lines.push(vec![TermRun::new(format!("{other}: command not found")).fg(AnsiColor::BrightRed)].into());
                                    exit.set(Some(127));
                                }
                            }
                        },
                    }
                },
            }
            Terminal {
                output: transcript(),
                label: name!("Terminal output, disconnected"),
                state: TermState::Disconnected,
                session: "bash".to_owned(),
                exit: 1,
                cursor: CursorStyle::Steady,
                onsubmit: move |_| {},
            }
        }
    }
}

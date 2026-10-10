use fc_dioxus::prelude::*;

// example:start
/// A contained composer with tools, attachments (one failed), and a send button that
/// becomes Stop while a reply is generating. Enter sends; Shift+Enter adds a line.
#[component]
pub fn States() -> Element {
    let mut draft = use_signal(|| "Summarise the thread".to_owned());
    let mut running = use_signal(|| false);
    let mut sent = use_signal(Vec::<String>::new);
    let mut attachments = use_signal(|| {
        vec![
            Attachment {
                name: "notes.md".into(),
                size: 2048,
                error: None,
            },
            Attachment {
                name: "a-very-long-file-name-that-is-truncated.pdf".into(),
                size: 0,
                error: Some("Could not read the file".into()),
            },
        ]
    });
    let mut web = use_signal(|| true);
    rsx! {
        Composer { label: name!("Message"), placeholder: "Ask anything…", contained: true,
            value: ReadSignal::from(draft), oninput: move |v| draft.set(v),
            running: running(), onstop: move |()| running.set(false),
            onsubmit: move |(text, _files): (String, Vec<Attachment>)| { sent.write().push(text); attachments.set(vec![]); running.set(true); },
            attachments: attachments(), onremove: move |i: usize| { attachments.write().remove(i); },
            onfiles: move |files: Vec<dioxus::html::FileData>| {
                attachments.write().extend(files.iter().map(|f| Attachment { name: f.name(), size: f.size(), error: None }));
            },
            tools: rsx! {
                ControlGroup { name: GroupName::Label(name!("Message context")), layout: ControlLayout::Plain,
                    Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm,
                        onclick: move |_| attachments.write().push(Attachment { name: "screenshot.png".into(), size: 48_000, error: None }),
                        "Attach" }
                }
                ControlGroup { name: GroupName::Label(name!("Tools")),
                    Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, pressed: web(), onclick: move |_| web.toggle(), "Web search" }
                }
            },
        }
        if !sent().is_empty() {
            p { "Sent: {sent().join(\" / \")}" }
        }
    }
}

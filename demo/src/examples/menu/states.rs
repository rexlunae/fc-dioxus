use fc_dioxus::prelude::*;

// example:start
/// A menu button with groups, shortcuts, an unavailable item, a checkbox item, a
/// submenu, and a destructive item; and an area with its own right-click menu.
/// Arrows move, typing searches, Escape closes and returns focus.
#[component]
pub fn States() -> Element {
    const STACK: Bem = Bem::new("fc-stack");
    const CARD: Bem = Bem::new("fc-card");
    let mut wrap = use_signal(|| true);
    let mut last = use_signal(String::new);
    let entries = vec![
        MenuEntry::Group("Edit".into()),
        MenuEntry::Item {
            id: "cut".into(),
            label: "Cut".into(),
            shortcut: Some("⌘X".into()),
            danger: false,
            disabled: false,
        },
        MenuEntry::item("copy", "Copy"),
        MenuEntry::Item {
            id: "paste".into(),
            label: "Paste".into(),
            shortcut: None,
            danger: false,
            disabled: true,
        },
        MenuEntry::Separator,
        MenuEntry::Checkbox {
            id: "wrap".into(),
            label: "Wrap lines".into(),
            checked: wrap(),
            disabled: false,
        },
        MenuEntry::Submenu {
            id: "share".into(),
            label: "Share".into(),
        },
        MenuEntry::Item {
            id: "delete".into(),
            label: "Delete file".into(),
            shortcut: None,
            danger: true,
            disabled: false,
        },
    ];
    let select = move |id: String| {
        if id == "wrap" {
            wrap.toggle();
        }
        last.set(id);
    };
    rsx! {
        div { class: STACK.block(),
            MenuButton { label: name!("Actions"), entries: entries.clone(), onselect: select, variant: ButtonVariant::Secondary, "Actions" }
            ContextMenuArea { label: name!("File actions"), entries, onselect: select, class: CARD.block(),
                p { "Right-click here, or press the context menu key, for the same actions." }
            }
            p { role: "status", if !last().is_empty() { "Chose: {last}" } }
        }
    }
}

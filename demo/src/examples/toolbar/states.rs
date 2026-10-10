use fc_dioxus::prelude::*;

// example:start
/// A floating formatting toolbar, a vertical tool palette, and a docked action bar. Each
/// is one tab stop; arrows move within it.
#[component]
pub fn States() -> Element {
    const STACK: Bem = Bem::new("fc-stack");
    let mut bold = use_signal(|| true);
    let mut italic = use_signal(|| false);
    let mut tool = use_signal(|| "select");
    let reason = use_id();
    rsx! {
        div { class: STACK.block(),
            Toolbar { label: name!("Selection"), surface: ToolbarSurface::Floating,
                ToolbarItem { pressed: bold(), onclick: move |_| bold.toggle(), "Bold" }
                ToolbarItem { pressed: italic(), onclick: move |_| italic.toggle(), "Italic" }
                ToolbarDivider {}
                ToolbarItem { icon_label: name!("Copy"), "⧉" }
                ToolbarItem { icon_label: name!("Paste"), availability: Availability::Unavailable { reason: reason.clone() }, "📋" }
                ToolbarItem { icon_label: name!("Undo"), availability: Availability::Disabled, "↶" }
            }
            p { id: "{reason}", "The clipboard is empty." }
            Toolbar { label: name!("Canvas tools"), orientation: Orientation::Vertical, surface: ToolbarSurface::Floating,
                ToolbarItem { icon_label: name!("Select"), pressed: tool() == "select", onclick: move |_| tool.set("select"), "➤" }
                ToolbarItem { icon_label: name!("Draw"), pressed: tool() == "draw", onclick: move |_| tool.set("draw"), "✎" }
                ToolbarDivider {}
                ToolbarItem { icon_label: name!("Delete"), "×" }
            }
            Toolbar { label: name!("Message actions"), surface: ToolbarSurface::Docked,
                ToolbarItem { "Reply" }
                ToolbarItem { "Forward to everyone on the thread" }
                ToolbarItem { "Archive" }
                ToolbarItem { "Delete" }
            }
            Toolbar { label: name!("Export"),
                ToolbarItem { "⤓ " ToolbarLabel { "Download" } }
                ToolbarItem { "⎘ " ToolbarLabel { "Duplicate" } }
            }
        }
    }
}

use fc_dioxus::{
    core::platform::{DomBridge, ScrollBlock},
    prelude::*,
};

// example:start
/// Tones and sizes, an extended FAB, disabled and busy states, and a FAB that opens a menu.
#[component]
pub fn States() -> Element {
    const CLUSTER: Bem = Bem::new("fc-cluster");
    let menu = use_id();
    let reason = use_id();
    let names = [
        "New chat",
        "Compose",
        "Upload",
        "Add",
        "Record",
        "New conversation with a long translated name",
        "Unavailable",
        "Creating…",
        "Create",
        "Back to top",
    ]
    .map(AccessibleName::try_from);
    let [Ok(new_chat), Ok(compose), Ok(upload), Ok(add), Ok(record), Ok(long), Ok(unavailable), Ok(creating), Ok(create), Ok(top)] =
        names
    else {
        return rsx! {};
    };
    rsx! {
        div { class: CLUSTER.block(),
            Fab { label: new_chat.clone(), icon: rsx! { "+" } }
            Fab { label: compose, tone: FabTone::Primary, icon: rsx! { "✎" } }
            Fab { label: upload, tone: FabTone::Surface, icon: rsx! { "↑" } }
            Fab { label: add, size: FabSize::Sm, icon: rsx! { "+" } }
            Fab { label: record, size: FabSize::Lg, lowered: true, icon: rsx! { "●" } }
        }
        div { class: CLUSTER.block(),
            Fab { label: long, extended: true, icon: rsx! { "+" } }
        }
        div { class: CLUSTER.block(),
            Fab { label: new_chat, availability: Availability::Disabled, icon: rsx! { "+" } }
            Fab { label: unavailable, extended: true, availability: Availability::Unavailable { reason: reason.clone() }, icon: rsx! { "+" } }
            Fab { label: creating, extended: true, busy: true, icon: rsx! { "+" } }
            span { id: "{reason}", "Your plan has reached its conversation limit." }
        }
        div { class: CLUSTER.block(),
            Fab { label: create, menu: menu.clone(), icon: rsx! { "+" } }
            FabMenu { id: menu,
                FabMenuItem { "Document" }
                FabMenuItem { "Folder" }
                FabMenuItem { availability: Availability::Disabled, "Shared drive" }
            }
        }
        // Docked to the viewport's end corner for the whole page.
        Fab { label: top, fixed: true, size: FabSize::Sm, tone: FabTone::Surface, icon: rsx! { "↑" },
            onclick: move |_| async move {
                if let Ok(id) = Id::try_from("top") { let _ = DomBridge.scroll_into_view(&id, ScrollBlock::Start).await; }
            },
        }
    }
}

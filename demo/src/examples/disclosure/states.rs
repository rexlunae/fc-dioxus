use fc_dioxus::prelude::*;

// example:start
/// Closed and open sections, plain and boxed. The browser handles toggling and keys.
#[component]
pub fn States() -> Element {
    const STACK: Bem = Bem::new("fc-stack");
    rsx! {
        div { class: STACK.block(),
            Disclosure { summary: rsx! { "Closed" }, "Hidden content." }
            Disclosure { summary: rsx! { "Open" }, open: true, "Shown content." }
            Disclosure { summary: rsx! { "Boxed, open" }, open: true, boxed: true, "A card per item." }
            Disclosure { summary: rsx! { "Boxed, closed" }, boxed: true, "Hidden." }
        }
    }
}

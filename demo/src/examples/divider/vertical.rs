use fc_dioxus::prelude::*;

// example:start
/// An unfocusable separator inside an upstream layout row.
#[component]
pub fn Vertical() -> Element {
    const ROW: Bem = Bem::new("fc-cluster");
    rsx! {
        div { class: ROW.block(),
            span { "Before" }
            Divider { orientation: Orientation::Vertical }
            span { "After" }
        }
    }
}

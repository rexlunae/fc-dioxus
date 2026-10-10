use fc_dioxus::prelude::*;

// example:start
/// Both insets also remain reachable in the vertical orientation.
#[component]
pub fn VerticalInsets() -> Element {
    const ROW: Bem = Bem::new("fc-cluster");
    rsx! {
        div { class: ROW.block(),
            span { "Start" }
            Divider { orientation: Orientation::Vertical, inset: DividerInset::Inset }
            span { "Middle" }
            Divider { orientation: Orientation::Vertical, inset: DividerInset::Middle }
            span { "End" }
        }
    }
}

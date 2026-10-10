use fc_dioxus::prelude::*;

// example:start
/// Native thematic breaks with each supported inset.
#[component]
pub fn Horizontal() -> Element {
    rsx! {
        div {
            p { "Above a full divider" }
            Divider {}
            p { "Above an inset divider" }
            Divider { inset: DividerInset::Inset }
            p { "Above a middle divider" }
            Divider { inset: DividerInset::Middle }
            p { "Below" }
        }
    }
}

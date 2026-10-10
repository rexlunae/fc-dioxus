use fc_dioxus::prelude::*;

// example:start
/// Columns with a numeric value, divided rows, and the stacked layout.
#[component]
pub fn States() -> Element {
    const STACK: Bem = Bem::new("fc-stack");
    rsx! {
        div { class: STACK.block(),
            DescriptionList {
                DescriptionItem { term: rsx! { "Model" }, "Large" }
                DescriptionItem { term: rsx! { "Duration" }, numeric: true, "14.2 s" }
                DescriptionItem { term: rsx! { "Commit" }, code { "9f2c1e04b7a3d58e6f1c2b9a0d4e7f3c8b1a6d5e" } }
            }
            DescriptionList { divided: true,
                DescriptionItem { term: rsx! { "Owner" }, "Ada" }
                DescriptionItem { term: rsx! { "A term long enough to wrap in its column" }, "Value" }
            }
            DescriptionList { layout: DlLayout::Stacked,
                DescriptionItem { term: rsx! { "Region" }, "eu-west-1" }
                DescriptionItem { term: rsx! { "Created" }, "6 Oct 2026" }
            }
        }
    }
}

use fc_dioxus::prelude::*;

// example:start
/// Underlined, segmented, and vertical tab lists. Arrows move between tabs (Left moves
/// forward in right-to-left text), Home and End jump, and the unavailable tab is skipped.
#[component]
pub fn States() -> Element {
    const STACK: Bem = Bem::new("fc-stack");
    let reason = use_id();
    rsx! {
        div { class: STACK.block(),
            for (label, orientation, segmented) in [(name!("Answer view"), Orientation::Horizontal, false),
                (name!("Range"), Orientation::Horizontal, true), (name!("Settings"), Orientation::Vertical, false)]
            {
                Tabs { label, orientation, segmented, default_value: "preview".to_owned(),
                    tabs: rsx! {
                        Tab { value: "preview", "Preview" }
                        Tab { value: "problems", count: 2, "Problems " }
                        Tab { value: "deploy", availability: Availability::Unavailable { reason: reason.clone() }, "Deploy" }
                    },
                    TabPanel { value: "preview", "The rendered answer." }
                    TabPanel { value: "problems", "Two problems: a missing alt text and an unlabelled field." }
                    TabPanel { value: "deploy", "Deployment settings." }
                }
            }
            p { id: "{reason}", "Deploying needs a connected environment." }
            Tabs { label: name!("Manual activation"), activation: Activation::Manual, default_value: "summary".to_owned(),
                tabs: rsx! { Tab { value: "summary", "Summary" } Tab { value: "raw", "Raw output" } },
                TabPanel { value: "summary", "Arrow keys move focus; Enter or Space shows the panel." }
                TabPanel { value: "raw", "Panels that fetch on selection wait for an explicit choice." }
            }
        }
    }
}

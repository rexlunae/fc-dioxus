use fc_dioxus::prelude::*;

// example:start
/// One choice, several choices, and an unanswered required choice.
#[component]
pub fn States() -> Element {
    const STACK: Bem = Bem::new("fc-stack");
    let unit_name = use_id();
    let keep_name = use_id();
    let mut unit = use_signal(|| "c".to_owned());
    let mut sources = use_signal(|| vec!["web".to_owned(), "files".to_owned()]);
    let names = ["Units", "Sources", "Retention"].map(AccessibleName::try_from);
    let [Ok(units), Ok(source), Ok(retention)] = names else {
        return rsx! {};
    };
    rsx! {
        div { class: STACK.block(),
            Segmented { legend: units,
                segments: Segments::Single { name: unit_name, value: Some(unit()), onchange: Some(EventHandler::new(move |v| unit.set(v))) },
                Segment { value: "c", "Celsius" }
                Segment { value: "f", "Fahrenheit" }
                Segment { value: "k", "Kelvin" }
            }
            Segmented { legend: source,
                segments: Segments::Multiple { values: sources(), onchange: Some(EventHandler::new(move |v| sources.set(v))) },
                Segment { value: "web", "Web" }
                Segment { value: "files", "Files" }
                Segment { value: "email", availability: Availability::Disabled, "Email" }
            }
            Segmented { legend: retention, invalid: true, required: true,
                segments: Segments::Single { name: keep_name, value: None, onchange: None },
                Segment { value: "forever", "Keep every conversation indefinitely" }
                Segment { value: "30d", "Delete after thirty days" }
            }
        }
    }
}

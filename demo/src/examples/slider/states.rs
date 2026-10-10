use fc_dioxus::prelude::*;

// example:start
/// A stepped slider with its value and marks, a plain one, an invalid one, and a disabled
/// one. The filled track follows the value.
#[component]
pub fn States() -> Element {
    const STACK: Bem = Bem::new("fc-stack");
    let mut temperature = use_signal(|| 0.7);
    rsx! {
        div { class: STACK.block(),
            Slider { label: name!("Temperature"), min: 0.0, max: 2.0, step: 0.1, show_value: true,
                value: ReadSignal::from(temperature), oninput: move |v| temperature.set(v),
                marks: vec!["0".into(), "1".into(), "2".into()] }
            Slider { label: name!("Volume"), default_value: 40.0 }
            Slider { label: name!("Budget"), default_value: 90.0, invalid: true, show_value: true }
            Slider { label: name!("Locked"), default_value: 60.0, availability: Availability::Disabled, show_value: true }
        }
    }
}

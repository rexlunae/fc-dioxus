//! A native range input with a filled track and visible value, from slider.css. The fill
//! is a port of upstream's `slider.ts`.
use crate::{
    core::{
        assets,
        attrs::{availability_attributes, component_attributes},
        measure::{custom_property, CustomProperty},
    },
    prelude::*,
    theme::use_stylesheets,
};
const BEM: Bem = Bem::new("fc-slider");

/// The share of the range `value` covers, as upstream's `sliderPercent` computes it: a
/// degenerate range is drawn empty rather than dividing by zero.
///
/// ```
/// use fc_dioxus::components::slider::slider_percent;
/// assert_eq!(slider_percent(0.7, 0.0, 2.0).get(), 35.0);
/// assert_eq!(slider_percent(5.0, 5.0, 5.0).get(), 0.0);
/// assert_eq!(slider_percent(-1.0, 0.0, 10.0).get(), 0.0);
/// ```
#[must_use]
pub fn slider_percent(value: f64, min: f64, max: f64) -> Percent {
    let span = max - min;
    if span > 0.0 {
        Percent::new((value - min) / span * 100.0)
    } else {
        Percent::new(0.0)
    }
}

/// A native `<input type="range">`, which already has the whole keyboard contract and
/// announces its own value. The visible `<output>` is not a live region, so a step is
/// never announced twice.
#[component]
pub fn Slider(
    label: AccessibleName,
    #[props(default = 0.0)] min: f64,
    #[props(default = 100.0)] max: f64,
    #[props(default = 1.0)] step: f64,
    #[props(default)] value: Option<ReadSignal<f64>>,
    #[props(default)] default_value: f64,
    #[props(default)] oninput: Option<EventHandler<f64>>,
    /// Show the value beside the track.
    #[props(default)]
    show_value: bool,
    /// Labels under the track, e.g. the ends and the middle. Decorative.
    #[props(default)]
    marks: Vec<String>,
    #[props(default)] invalid: bool,
    #[props(default)] availability: Availability,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let state = use_controllable(value, || default_value, oninput);
    let id = use_id();
    let current = state.get();
    let attributes = component_attributes(
        attributes,
        ClassList::new(BEM.block()).extend(class.as_deref()),
        &[],
    );
    let mut input = availability_attributes(&availability);
    input.push(custom_property(
        CustomProperty::SliderPct,
        format!("{}%", slider_percent(current, min, max)),
    ));
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_SLIDER_CSS } }
        div { ..attributes,
            input { class: BEM.element("input"), r#type: "range", id: "{id}", min: "{min}", max: "{max}", step: "{step}",
                value: "{current}", aria_label: label.to_string(), aria_invalid: invalid.then_some("true"),
                oninput: move |event| if let Ok(value) = event.value().parse::<f64>() { state.set(value); },
                ..input,
            }
            if show_value { output { class: BEM.element("value"), ..vec![Attribute::new("for", id.to_string(), None, false)], "{current}" } }
            if !marks.is_empty() {
                ol { class: BEM.element("marks"), aria_hidden: "true",
                    for mark in marks { li { "{mark}" } }
                }
            }
        }
    }
}

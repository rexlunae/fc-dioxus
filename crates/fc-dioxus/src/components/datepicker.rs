//! A month grid for browsing and choosing dates, and a segmented time entry, from
//! datepicker.css. The grid ports upstream's `datepicker.ts`.
//!
//! The calendar is for browsing. Anyone who knows the date types it faster into a native
//! `<input type="date">`, which is also the path with no script, so offer that input and
//! open the calendar beside it rather than replacing it.
use crate::{
    behavior::date::{days_in_month, key_target, leading_blanks, Date},
    core::{
        assets,
        attrs::{
            component_attributes, date_attribute, date_value_attribute, datepicker_nav_attribute,
        },
        platform::DomBridge,
    },
    prelude::*,
    theme::use_stylesheets,
};
const BEM: Bem = Bem::new("fc-datepicker");
const CHEVRON: Bem = Bem::new("fc-datepicker__chevron");
const BTN: Bem = Bem::new("fc-btn");
const TIME: Bem = Bem::new("fc-timepicker");
const INPUT: Bem = Bem::new("fc-input");
const FIELD_LABEL: Bem = Bem::new("fc-field__label");
fc_modifier! { enum ChevronSide for CHEVRON { Plain => None, Prev => "prev", Next => "next" } }

/// The words a calendar shows. Rust has no locale data, so names come from the app; the
/// default is English.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateNames {
    /// January to December.
    pub months: [String; 12],
    /// Sunday to Saturday, in full: column abbreviations and day names.
    pub weekdays: [String; 7],
    /// Sunday to Saturday, as drawn in the column headers.
    pub narrow: [String; 7],
    /// Names the previous-month button.
    pub previous: String,
    /// Names the next-month button.
    pub next: String,
}
impl Default for DateNames {
    fn default() -> Self {
        let owned = |names: &[&str]| names.iter().map(|n| (*n).to_owned()).collect::<Vec<_>>();
        let months = owned(&[
            "January",
            "February",
            "March",
            "April",
            "May",
            "June",
            "July",
            "August",
            "September",
            "October",
            "November",
            "December",
        ]);
        let weekdays = owned(&[
            "Sunday",
            "Monday",
            "Tuesday",
            "Wednesday",
            "Thursday",
            "Friday",
            "Saturday",
        ]);
        Self {
            months: std::array::from_fn(|i| months.get(i).cloned().unwrap_or_default()),
            narrow: std::array::from_fn(|i| {
                weekdays
                    .get(i)
                    .and_then(|w| w.get(..1))
                    .unwrap_or_default()
                    .to_owned()
            }),
            weekdays: std::array::from_fn(|i| weekdays.get(i).cloned().unwrap_or_default()),
            previous: "Previous month".to_owned(),
            next: "Next month".to_owned(),
        }
    }
}
impl DateNames {
    fn month(&self, date: Date) -> &str {
        self.months
            .get(usize::from(date.month()) - 1)
            .map_or("", String::as_str)
    }
    fn weekday(&self, index: u8) -> &str {
        self.weekdays
            .get(usize::from(index % 7))
            .map_or("", String::as_str)
    }
    /// "Wednesday, October 21, 2026": a day button's accessible name.
    fn full(&self, date: Date) -> String {
        format!(
            "{}, {} {}, {}",
            self.weekday(date.weekday()),
            self.month(date),
            date.day(),
            date.year()
        )
    }
}

/// A month grid on the WAI-ARIA date grid pattern: one tab stop, arrows by day (swapped
/// in RTL) and week, Home and End to the week's ends, Page Up and Down by month (Shift
/// for a year), Enter or Space to choose. Unavailable days stay focusable so they can be
/// discovered, but cannot be chosen.
#[component]
pub fn DatePicker(
    label: AccessibleName,
    #[props(default)] value: Option<ReadSignal<Option<Date>>>,
    #[props(default)] default_value: Option<Date>,
    #[props(default)] onchange: Option<EventHandler<Date>>,
    #[props(default)] min: Option<Date>,
    #[props(default)] max: Option<Date>,
    /// Today, marked `aria-current="date"`. The library reads no clock, so pass it in.
    #[props(default)]
    today: Option<Date>,
    /// Further unavailable days, e.g. weekends.
    #[props(default)]
    unavailable: Option<Callback<Date, bool>>,
    /// 0 = Sunday … 6 = Saturday.
    #[props(default)]
    first_day_of_week: u8,
    #[props(default)] names: DateNames,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let state = use_controllable(
        value,
        || default_value,
        onchange.map(|onchange| {
            EventHandler::new(move |date: Option<Date>| {
                if let Some(date) = date {
                    onchange.call(date);
                }
            })
        }),
    );
    let selected = state.get();
    let clamp = move |date: Date| match (min, max) {
        (Some(min), _) if date < min => min,
        (_, Some(max)) if date > max => max,
        _ => date,
    };
    let fallback = today.or(min).or(Date::new(2000, 1, 1).ok());
    let mut focus = use_signal(|| selected.or(fallback).map(clamp));
    let mut pending = use_signal(|| false);
    let id = use_id();
    let rtl = use_direction() == Direction::Rtl;
    let blocked = move |date: Date| {
        min.is_some_and(|m| date < m)
            || max.is_some_and(|m| date > m)
            || unavailable.is_some_and(|u| u.call(date))
    };
    let Some(shown) = focus() else {
        return rsx! {};
    };
    // After a key moves focus into another month the grid re-renders; focus the new day
    // once it exists.
    let effect_id = id.clone();
    use_effect(move || {
        if pending() {
            pending.set(false);
            if let Ok(day) = Id::try_from(format!("{effect_id}-day")) {
                spawn(async move {
                    let _ = DomBridge.focus(&day).await;
                });
            }
        }
    });
    let first = shown.first_of_month();
    let blanks = leading_blanks(first, first_day_of_week);
    let days = days_in_month(first.year(), first.month());
    let prev_disabled = min.is_some_and(|m| first.add_days(-1).is_some_and(|d| d < m));
    let next_disabled = max.is_some_and(|m| first.add_months(1).is_some_and(|d| d > m));
    let title_id = id.suffixed("title");
    let mut attributes = component_attributes(
        attributes,
        ClassList::new(BEM.block()).extend(class.as_deref()),
        &["aria-label"],
    );
    attributes.extend(date_value_attribute(selected.map(|d| d.to_string())));
    let nav_class = ClassList::new(BTN.block())
        .with(BTN.modifier_raw("ghost"))
        .with(BTN.modifier_raw("icon"))
        .with(BTN.modifier_raw("sm"))
        .to_string();
    let mut page = move |delta: i32| {
        if let Some(date) = focus().and_then(|d| d.add_months(delta)) {
            focus.set(Some(clamp(date)));
        }
    };
    let cells: Vec<Option<Date>> = (0..blanks)
        .map(|_| None)
        .chain((1..=days).map(|d| Date::new(first.year(), first.month(), d).ok()))
        .collect();
    let rows: Vec<Vec<Option<Date>>> = cells
        .chunks(7)
        .map(|row| {
            let mut row = row.to_vec();
            row.resize(7, None);
            row
        })
        .collect();
    let names_for_cells = names.clone();
    rsx! {
        if use_stylesheets() {
            document::Stylesheet { href: assets::COMPONENTS_BUTTON_CSS }
            document::Stylesheet { href: assets::COMPONENTS_DATEPICKER_CSS }
        }
        div { aria_label: label.to_string(), ..attributes,
            div { class: BEM.element("header"),
                button { class: nav_class.clone(), r#type: "button", aria_label: "{names.previous}",
                    aria_disabled: prev_disabled.then_some("true"),
                    onclick: move |_| if !prev_disabled { page(-1); },
                    ..crate::spread![datepicker_nav_attribute(false)],
                    span { class: ClassList::new(CHEVRON.block()).with_modifier(CHEVRON, ChevronSide::Prev).to_string(), aria_hidden: "true" }
                }
                // Announces the new month when the buttons or Page keys change it.
                h2 { class: BEM.element("title"), id: "{title_id}", aria_live: "polite", "{names.month(first)} {first.year()}" }
                button { class: nav_class, r#type: "button", aria_label: "{names.next}",
                    aria_disabled: next_disabled.then_some("true"),
                    onclick: move |_| if !next_disabled { page(1); },
                    ..crate::spread![datepicker_nav_attribute(true)],
                    span { class: ClassList::new(CHEVRON.block()).with_modifier(CHEVRON, ChevronSide::Next).to_string(), aria_hidden: "true" }
                }
            }
            table { class: BEM.element("grid"), role: "grid", aria_labelledby: "{title_id}",
                thead { tr {
                    for column in 0..7_u8 {
                        th { key: "{column}", class: BEM.element("weekday"), scope: "col",
                            abbr: "{names.weekday(column + first_day_of_week)}",
                            {names.narrow.get(usize::from((column + first_day_of_week) % 7)).cloned().unwrap_or_default()}
                        }
                    }
                } }
                tbody {
                    for (index, row) in rows.into_iter().enumerate() {
                        tr { key: "{index}",
                            for (column, cell) in row.into_iter().enumerate() {
                                match cell {
                                    None => rsx! { td { key: "{column}", class: BEM.element("cell") } },
                                    Some(date) => {
                                        let is_focus = date == shown;
                                        let disabled = blocked(date);
                                        let iso = date.to_string();
                                        let label = names_for_cells.full(date);
                                        rsx! {
                                            td { key: "{column}", class: BEM.element("cell"), role: "gridcell",
                                                aria_selected: (selected == Some(date)).to_string(),
                                                button { class: BEM.element("day"), r#type: "button",
                                                    id: if is_focus { format!("{id}-day") } else { String::new() },
                                                    tabindex: if is_focus { "0" } else { "-1" },
                                                    aria_label: "{label}",
                                                    aria_current: (today == Some(date)).then_some("date"),
                                                    aria_disabled: disabled.then_some("true"),
                                                    onclick: move |_| if !disabled { focus.set(Some(date)); state.set(Some(date)); },
                                                    onkeydown: move |event| {
                                                        if let Some(next) = key_target(&event.key(), event.modifiers().shift(), rtl, date, first_day_of_week) {
                                                            event.prevent_default();
                                                            focus.set(Some(clamp(next)));
                                                            pending.set(true);
                                                        }
                                                    },
                                                    ..crate::spread![date_attribute(&iso)],
                                                    "{date.day()}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// A time of day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Time {
    /// 0–23.
    pub hour: u8,
    /// 0–59.
    pub minute: u8,
}

/// Hour and minute fields, with an AM/PM choice in twelve-hour mode. Each field takes
/// digits only and clamps to its range when it loses focus.
#[component]
pub fn TimePicker(
    legend: Element,
    #[props(default)] value: Option<ReadSignal<Time>>,
    #[props(default = Time { hour: 9, minute: 0 })] default_value: Time,
    #[props(default)] onchange: Option<EventHandler<Time>>,
    #[props(default)] twelve_hour: bool,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let state = use_controllable(value, || default_value, onchange);
    let time = state.get();
    let period_name = use_id();
    let attributes = component_attributes(
        attributes,
        ClassList::new(TIME.block()).extend(class.as_deref()),
        &[],
    );
    let unit_class = ClassList::new(INPUT.block())
        .with(TIME.element("unit"))
        .to_string();
    let shown_hour = if twelve_hour {
        (time.hour + 11) % 12 + 1
    } else {
        time.hour
    };
    let pm = time.hour >= 12;
    let set_hour = move |text: String| {
        let Ok(typed) = text.parse::<u8>() else {
            return;
        };
        let hour = if twelve_hour {
            let base = typed.clamp(1, 12) % 12;
            if state.get().hour >= 12 {
                base + 12
            } else {
                base
            }
        } else {
            typed.min(23)
        };
        state.set(Time {
            hour,
            ..state.get()
        });
    };
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_DATEPICKER_CSS } }
        fieldset { ..attributes,
            legend { class: FIELD_LABEL.block(), {legend} }
            input { class: unit_class.clone(), inputmode: "numeric", aria_label: "Hour", maxlength: "2",
                value: "{shown_hour:02}", onchange: move |event| set_hour(event.value()) }
            span { class: TIME.element("separator"), aria_hidden: "true", ":" }
            input { class: unit_class, inputmode: "numeric", aria_label: "Minute", maxlength: "2",
                value: "{time.minute:02}",
                onchange: move |event| if let Ok(minute) = event.value().parse::<u8>() { state.set(Time { minute: minute.min(59), ..state.get() }); } }
            if twelve_hour {
                Segmented { legend: crate::name!("Period"),
                    segments: Segments::Single { name: period_name, value: Some(if pm { "pm" } else { "am" }.to_owned()),
                        onchange: Some(EventHandler::new(move |period: String| {
                            let now = state.get();
                            let hour = match (period.as_str(), now.hour >= 12) {
                                ("pm", false) => now.hour + 12,
                                ("am", true) => now.hour - 12,
                                _ => now.hour,
                            };
                            state.set(Time { hour, ..now });
                        })) },
                    Segment { value: "am", "AM" }
                    Segment { value: "pm", "PM" }
                }
            }
        }
    }
}

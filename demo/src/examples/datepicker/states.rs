use fc_dioxus::prelude::*;

// example:start
/// A month grid with a selection and today marked, one that refuses dates before a
/// minimum and on weekends, the native inputs to offer beside them, and a time entry.
#[component]
pub fn States() -> Element {
    const STACK: Bem = Bem::new("fc-stack");
    const GRID: Bem = Bem::new("fc-grid-auto");
    let mut due = use_signal(|| Date::parse("2026-10-21").ok());
    let mut reminder = use_signal(|| Time {
        hour: 9,
        minute: 30,
    });
    let today = Date::parse("2026-10-10").ok();
    let weekend = Callback::new(|date: Date| date.weekday() == 0 || date.weekday() == 6);
    rsx! {
        div { class: STACK.block(),
            div { class: GRID.block(),
                DatePicker { label: name!("Due date"), today, value: ReadSignal::from(due), onchange: move |d| due.set(Some(d)) }
                DatePicker { label: name!("Delivery date"), today, min: Date::parse("2026-10-12").ok(), unavailable: weekend,
                    first_day_of_week: 1 }
            }
            p { role: "status", if let Some(due) = due() { "Due {due}" } }
            div { class: GRID.block(),
                Field { label: rsx! { "Due date" }, Input { r#type: InputType::Date, default_value: "2026-10-21" } }
                Field { label: rsx! { "Start time" }, Input { r#type: InputType::Time, default_value: "09:30" } }
                Field { label: rsx! { "Locked date" }, Input { r#type: InputType::Date, default_value: "2026-10-01", availability: Availability::Disabled } }
            }
            TimePicker { legend: rsx! { "Reminder" }, twelve_hour: true, value: ReadSignal::from(reminder), onchange: move |t| reminder.set(t) }
            p { role: "status", "Reminder at {reminder().hour:02}:{reminder().minute:02}" }
        }
    }
}

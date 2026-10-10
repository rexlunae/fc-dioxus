//! Calendar arithmetic at the edges of its range.
#![forbid(unsafe_code)]

use fc_dioxus::behavior::date::Date;

#[test]
fn offsets_past_the_range_are_none_rather_than_overflowing() {
    let Ok(day) = Date::new(2024, 2, 29) else {
        return assert!(Date::new(2024, 2, 29).is_ok());
    };
    assert_eq!(day.add_days(i64::MAX), None);
    assert_eq!(day.add_days(i64::MIN), None);
    assert_eq!(day.add_months(i32::MAX), None);
    assert_eq!(day.add_months(i32::MIN), None);
    assert_eq!(
        day.add_days(1).map(|d| d.to_string()),
        Some("2024-03-01".to_owned())
    );
    assert_eq!(
        day.add_months(12).map(|d| d.to_string()),
        Some("2025-02-28".to_owned())
    );
}

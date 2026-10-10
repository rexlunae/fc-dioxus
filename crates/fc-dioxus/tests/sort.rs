//! Table sorting, against the behaviour upstream's table.ts documents.
#![forbid(unsafe_code)]

use fc_dioxus::behavior::sort::{as_number, next_direction, order};
use fc_dioxus::prelude::SortDirection;

#[test]
fn numbers_sort_numerically_after_removing_grouping() {
    assert_eq!(order(&["100", "9", "10"], SortDirection::Ascending), [1, 2, 0]);
    assert_eq!(order(&["12,480", "88,012", "9"], SortDirection::Descending), [1, 0, 2]);
    assert_eq!(as_number("1 234"), Some(1234.0));
    assert_eq!(as_number(""), None);
    assert_eq!(as_number("NaN"), None);
}

#[test]
fn sorting_is_stable_and_text_ignores_case() {
    // "complete" twice keeps its original order.
    assert_eq!(order(&["complete", "running", "Complete"], SortDirection::Ascending), [0, 2, 1]);
    assert_eq!(order(&["b", "A", "c"], SortDirection::Ascending), [1, 0, 2]);
}

#[test]
fn a_press_flips_only_an_ascending_column() {
    assert_eq!(next_direction(None, 2), SortDirection::Ascending);
    assert_eq!(next_direction(Some((2, SortDirection::Ascending)), 2), SortDirection::Descending);
    assert_eq!(next_direction(Some((2, SortDirection::Descending)), 2), SortDirection::Ascending);
    assert_eq!(next_direction(Some((1, SortDirection::Ascending)), 2), SortDirection::Ascending);
}

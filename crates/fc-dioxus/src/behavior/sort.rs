//! Table sorting, ported from upstream's `table.ts`.
//!
//! Numbers compare numerically when both values parse, after removing grouping
//! separators, so "9, 10, 100" and "12,480" sort as numbers. Other text compares
//! naturally (digit runs by value) and without regard to case. Upstream uses the
//! browser's `Intl.Collator` in the table's language for text; this port has no locale
//! tailoring, so give accented or locale-sensitive columns an explicit sort key.

use std::cmp::Ordering;

/// Which way a column is sorted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SortDirection {
    /// Smallest first.
    Ascending,
    /// Largest first.
    Descending,
}
impl SortDirection {
    /// The `aria-sort` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ascending => "ascending",
            Self::Descending => "descending",
        }
    }
}

/// The direction a header press picks: a column sorted ascending flips to descending,
/// anything else starts ascending.
#[must_use]
pub fn next_direction(current: Option<(usize, SortDirection)>, column: usize) -> SortDirection {
    match current {
        Some((sorted, SortDirection::Ascending)) if sorted == column => SortDirection::Descending,
        _ => SortDirection::Ascending,
    }
}

/// A value as a number, ignoring spaces and commas used for grouping.
#[must_use]
pub fn as_number(value: &str) -> Option<f64> {
    let cleaned: String = value
        .chars()
        .filter(|c| !matches!(c, ',' | ' ' | '\u{a0}' | '\u{202f}') && !c.is_whitespace())
        .collect();
    if cleaned.is_empty() {
        return None;
    }
    cleaned.parse::<f64>().ok().filter(|n| n.is_finite())
}

/// Compare two sort keys as `table.ts` does: numerically when both are numbers.
///
/// ```
/// use fc_dioxus::behavior::sort::compare;
/// use std::cmp::Ordering;
/// assert_eq!(compare("9", "10"), Ordering::Less);
/// assert_eq!(compare("12,480", "9"), Ordering::Greater);
/// assert_eq!(compare("item 2", "item 10"), Ordering::Less);
/// assert_eq!(compare("apple", "Banana"), Ordering::Less);
/// ```
#[must_use]
pub fn compare(a: &str, b: &str) -> Ordering {
    match (as_number(a), as_number(b)) {
        (Some(x), Some(y)) => x.partial_cmp(&y).unwrap_or(Ordering::Equal),
        _ => natural(a, b),
    }
}

fn natural(a: &str, b: &str) -> Ordering {
    let mut left = a.chars().peekable();
    let mut right = b.chars().peekable();
    loop {
        match (left.peek().copied(), right.peek().copied()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) if x.is_ascii_digit() && y.is_ascii_digit() => {
                let take = |it: &mut std::iter::Peekable<std::str::Chars<'_>>| {
                    let mut digits = String::new();
                    while let Some(c) = it.peek().copied().filter(char::is_ascii_digit) {
                        digits.push(c);
                        it.next();
                    }
                    digits
                };
                let (x, y) = (take(&mut left), take(&mut right));
                let (xs, ys) = (x.trim_start_matches('0'), y.trim_start_matches('0'));
                let order = xs.len().cmp(&ys.len()).then_with(|| xs.cmp(ys));
                if order != Ordering::Equal {
                    return order;
                }
            }
            (Some(x), Some(y)) => {
                let order = x.to_lowercase().cmp(y.to_lowercase());
                if order != Ordering::Equal {
                    return order;
                }
                left.next();
                right.next();
            }
        }
    }
}

/// Indices of `keys` in sorted order. Stable: equal keys keep their order, so sorting by
/// one column after another groups within the earlier order.
#[must_use]
pub fn order(keys: &[&str], direction: SortDirection) -> Vec<usize> {
    let mut indices: Vec<usize> = (0..keys.len()).collect();
    indices.sort_by(|&a, &b| {
        let ordering = compare(
            keys.get(a).copied().unwrap_or(""),
            keys.get(b).copied().unwrap_or(""),
        );
        match direction {
            SortDirection::Ascending => ordering,
            SortDirection::Descending => ordering.reverse(),
        }
    });
    indices
}

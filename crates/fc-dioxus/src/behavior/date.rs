//! Calendar arithmetic for the date picker, ported from upstream's `datepicker.ts`.
//!
//! Dates are civil dates with no time zone, so no daylight-saving shift can turn
//! midnight into the previous day; they cross the API as ISO `YYYY-MM-DD`. Day counts
//! use Howard Hinnant's `days_from_civil`, exact over the whole proleptic Gregorian range
//! this type allows.

use std::fmt;

/// A calendar date. Construction validates it, so every `Date` exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Date {
    year: i32,
    month: u8,
    day: u8,
}

/// The text or numbers do not name a real date.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidDate;
impl fmt::Display for InvalidDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("expected a real date as YYYY-MM-DD")
    }
}
impl std::error::Error for InvalidDate {}

const fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Days in a month of a year.
#[must_use]
pub const fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        2 if is_leap(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

impl Date {
    /// A date from its parts.
    ///
    /// # Errors
    /// Returns `InvalidDate` for a month outside 1–12 or a day the month lacks, or a year
    /// outside 1–9999, which ISO strings cannot write in four digits.
    pub fn new(year: i32, month: u8, day: u8) -> Result<Self, InvalidDate> {
        if (1..=9999).contains(&year)
            && (1..=12).contains(&month)
            && day >= 1
            && day <= days_in_month(year, month)
        {
            Ok(Self { year, month, day })
        } else {
            Err(InvalidDate)
        }
    }
    /// Parse `YYYY-MM-DD`.
    ///
    /// # Errors
    /// Returns `InvalidDate` for any other shape or a date that does not exist.
    pub fn parse(iso: &str) -> Result<Self, InvalidDate> {
        let mut parts = iso.split('-');
        let (Some(y), Some(m), Some(d), None) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            return Err(InvalidDate);
        };
        if y.len() != 4 || m.len() != 2 || d.len() != 2 {
            return Err(InvalidDate);
        }
        let number = |s: &str| s.parse::<u16>().map_err(|_| InvalidDate);
        let year = i32::from(number(y)?);
        let month = u8::try_from(number(m)?).map_err(|_| InvalidDate)?;
        let day = u8::try_from(number(d)?).map_err(|_| InvalidDate)?;
        Self::new(year, month, day)
    }
    /// The year.
    #[must_use]
    pub const fn year(self) -> i32 {
        self.year
    }
    /// The month, 1–12.
    #[must_use]
    pub const fn month(self) -> u8 {
        self.month
    }
    /// The day of the month.
    #[must_use]
    pub const fn day(self) -> u8 {
        self.day
    }
    /// Days since 1970-01-01.
    #[must_use]
    pub fn to_days(self) -> i64 {
        let y = i64::from(self.year) - i64::from(self.month <= 2);
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let m = i64::from(self.month);
        let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + i64::from(self.day) - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }
    /// The date `days` since 1970-01-01, or `None` outside years 1–9999.
    #[must_use]
    pub fn from_days(days: i64) -> Option<Self> {
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = doy - (153 * mp + 2) / 5 + 1;
        let month = if mp < 10 { mp + 3 } else { mp - 9 };
        let year = yoe + era * 400 + i64::from(month <= 2);
        Self::new(
            i32::try_from(year).ok()?,
            u8::try_from(month).ok()?,
            u8::try_from(day).ok()?,
        )
        .ok()
    }
    /// The date `delta` days away, or `None` past the representable range.
    #[must_use]
    pub fn add_days(self, delta: i64) -> Option<Self> {
        Self::from_days(self.to_days() + delta)
    }
    /// Day of the week, 0 = Sunday … 6 = Saturday.
    #[must_use]
    pub fn weekday(self) -> u8 {
        // 1970-01-01 was a Thursday.
        u8::try_from((self.to_days() + 4).rem_euclid(7)).unwrap_or(0)
    }
    /// The same day `delta` months away, clamped to that month's length, as upstream's
    /// `addMonths`.
    ///
    /// ```
    /// use fc_dioxus::behavior::date::Date;
    /// let Ok(jan31) = Date::parse("2026-01-31") else { return };
    /// assert_eq!(jan31.add_months(1).map(|d| d.to_string()), Some("2026-02-28".into()));
    /// assert_eq!(jan31.add_months(-13).map(|d| d.to_string()), Some("2024-12-31".into()));
    /// ```
    #[must_use]
    pub fn add_months(self, delta: i32) -> Option<Self> {
        let index = self.year * 12 + i32::from(self.month) - 1 + delta;
        let year = index.div_euclid(12);
        let month = u8::try_from(index.rem_euclid(12) + 1).ok()?;
        let day = self.day.min(days_in_month(year, month));
        Self::new(year, month, day).ok()
    }
    /// The first of this date's month.
    #[must_use]
    pub fn first_of_month(self) -> Self {
        Self { day: 1, ..self }
    }
}
impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// How many blank cells come before the first of the month, given the first day of the
/// week (0 = Sunday).
#[must_use]
pub fn leading_blanks(first_of_month: Date, first_day_of_week: u8) -> u8 {
    (first_of_month.weekday() + 7 - first_day_of_week % 7) % 7
}

/// Where a key moves focus from `current`, as upstream's date grid does: arrows by a day
/// (swapped in RTL) or a week, Home and End to the ends of the week, Page Up and Down by
/// a month, or a year with Shift.
#[must_use]
pub fn key_target(
    key: &dioxus::prelude::Key,
    shift: bool,
    rtl: bool,
    current: Date,
    first_day_of_week: u8,
) -> Option<Date> {
    use dioxus::prelude::Key;
    let weekday = i64::from((current.weekday() + 7 - first_day_of_week % 7) % 7);
    match key {
        Key::ArrowRight => current.add_days(if rtl { -1 } else { 1 }),
        Key::ArrowLeft => current.add_days(if rtl { 1 } else { -1 }),
        Key::ArrowDown => current.add_days(7),
        Key::ArrowUp => current.add_days(-7),
        Key::Home => current.add_days(-weekday),
        Key::End => current.add_days(6 - weekday),
        Key::PageDown => current.add_months(if shift { 12 } else { 1 }),
        Key::PageUp => current.add_months(if shift { -12 } else { -1 }),
        _ => None,
    }
}

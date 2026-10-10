//! Window splitter arithmetic, ported from upstream's `splitter.ts`: clamping and
//! rounding, what each key does, and how far a drag moves the bar.

use dioxus::prelude::Key;

/// The unit a splitter's value is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SplitUnit {
    /// A share of the split, so the layout survives a window resize.
    Percent,
    /// CSS pixels of the element's border box, for an edge handle.
    Px,
}
impl SplitUnit {
    /// The CSS unit suffix.
    #[must_use]
    pub const fn suffix(self) -> &'static str {
        match self {
            Self::Percent => "%",
            Self::Px => "px",
        }
    }
    /// The default arrow step: 5 for a percentage, 16 for pixels.
    #[must_use]
    pub const fn step(self) -> f64 {
        match self {
            Self::Percent => 5.0,
            Self::Px => 16.0,
        }
    }
}

/// Clamp to the bounds and round: tenths of a percent, whole pixels.
///
/// ```
/// use fc_dioxus::behavior::splitter::{settle, SplitUnit};
/// assert_eq!(settle(35.04, 15.0, 70.0, SplitUnit::Percent), 35.0);
/// assert_eq!(settle(90.0, 15.0, 70.0, SplitUnit::Percent), 70.0);
/// assert_eq!(settle(191.6, 120.0, 288.0, SplitUnit::Px), 192.0);
/// ```
#[must_use]
pub fn settle(value: f64, min: f64, max: f64, unit: SplitUnit) -> f64 {
    let clamped = value.clamp(min, max.max(min));
    match unit {
        SplitUnit::Percent => (clamped * 10.0).round() / 10.0,
        SplitUnit::Px => clamped.round(),
    }
}

/// Everything a key press needs to know about the bar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KeyContext {
    /// The bar moves along the inline axis (it is vertical).
    pub inline: bool,
    /// The bar sits in right-to-left text.
    pub rtl: bool,
    /// +1 when moving the bar toward the end of its axis grows the controlled pane.
    pub growth: f64,
    /// The current value.
    pub current: f64,
    /// Lower bound.
    pub min: f64,
    /// Upper bound.
    pub max: f64,
    /// One arrow press; Shift moves ten times as far.
    pub step: f64,
    /// The size Enter restores from the minimum, if one is remembered.
    pub restore: Option<f64>,
}

/// The value a key asks for, and the size to remember for Enter, or `None` for keys the
/// splitter leaves alone. Arrows move the bar, not the value: whether that grows the pane
/// depends on which side of the bar it is on, and in RTL Right is the inline start.
///
/// ```
/// use dioxus::prelude::Key;
/// use fc_dioxus::behavior::splitter::{key_value, KeyContext};
/// let bar = KeyContext { inline: true, rtl: false, growth: 1.0, current: 35.0, min: 15.0, max: 70.0, step: 5.0, restore: None };
/// assert_eq!(key_value(&Key::ArrowRight, false, bar), Some((40.0, None)));
/// assert_eq!(key_value(&Key::ArrowRight, true, bar), Some((85.0, None)));
/// assert_eq!(key_value(&Key::ArrowRight, false, KeyContext { rtl: true, ..bar }), Some((30.0, None)));
/// assert_eq!(key_value(&Key::Enter, false, bar), Some((15.0, Some(35.0))));
/// assert_eq!(key_value(&Key::Enter, false, KeyContext { current: 15.0, restore: Some(35.0), ..bar }), Some((35.0, None)));
/// assert_eq!(key_value(&Key::ArrowDown, false, bar), None);
/// ```
#[must_use]
pub fn key_value(key: &Key, shift: bool, bar: KeyContext) -> Option<(f64, Option<f64>)> {
    let step = bar.step * if shift { 10.0 } else { 1.0 };
    let toward_end = match (bar.inline, key) {
        (true, Key::ArrowRight) => {
            if bar.rtl {
                -1.0
            } else {
                1.0
            }
        }
        (true, Key::ArrowLeft) => {
            if bar.rtl {
                1.0
            } else {
                -1.0
            }
        }
        (false, Key::ArrowDown) => 1.0,
        (false, Key::ArrowUp) => -1.0,
        _ => 0.0,
    };
    if toward_end != 0.0 {
        return Some((bar.current + toward_end * bar.growth * step, None));
    }
    match key {
        Key::Home => Some((bar.min, None)),
        Key::End => Some((bar.max, None)),
        Key::Enter if bar.current > bar.min => Some((bar.min, Some(bar.current))),
        Key::Enter => Some((bar.restore.unwrap_or(bar.max), None)),
        _ => None,
    }
}

/// The value after dragging `delta` pixels from where a drag began at `start`.
/// `per_pixel` is 100 divided by the split's size for a percentage, 1 for pixels.
#[must_use]
pub fn drag_value(start: f64, delta: f64, per_pixel: f64, growth: f64, rtl_inline: bool) -> f64 {
    let sign = growth * if rtl_inline { -1.0 } else { 1.0 };
    start + sign * delta * per_pixel
}

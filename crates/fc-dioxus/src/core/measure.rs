//! The only inline styles this crate writes: custom properties upstream reads from markup.

use dioxus::prelude::*;
use std::fmt;

/// A percentage, clamped to `0..=100` when constructed so the drawn value and the ARIA
/// value can never disagree or leave the range.
///
/// ```
/// use fc_dioxus::prelude::Percent;
/// assert_eq!(Percent::new(140.0).get(), 100.0);
/// assert_eq!(Percent::new(f64::NAN).get(), 0.0);
/// ```
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct Percent(f64);
impl Percent {
    /// Clamp a value to `0..=100`; NaN becomes zero rather than poisoning the style.
    #[must_use]
    pub fn new(value: f64) -> Self {
        if value.is_nan() {
            Self(0.0)
        } else {
            Self(value.clamp(0.0, 100.0))
        }
    }
    /// Express `value` out of `max` as a percentage; a non-positive `max` is empty.
    #[must_use]
    pub fn of(value: f64, max: f64) -> Self {
        if max > 0.0 {
            Self::new(value / max * 100.0)
        } else {
            Self(0.0)
        }
    }
    /// The clamped value.
    #[must_use]
    pub const fn get(self) -> f64 {
        self.0
    }
}
impl fmt::Display for Percent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Whole numbers print without a fraction, so markup matches upstream's fixtures.
        if self.0.fract() == 0.0 {
            write!(f, "{:.0}", self.0)
        } else {
            write!(f, "{}", self.0)
        }
    }
}

/// The custom properties upstream's CSS reads from markup, and nothing else.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CustomProperty {
    /// Progress ring fill, 0 to 100.
    ProgressValue,
    /// A tool call's determinate progress, as a percentage length.
    ToolProgress,
    /// The filled part of a slider's track, as a percentage length.
    SliderPct,
    /// A split pane's or resizable element's size.
    SplitSize,
    /// A stacked split's block size.
    SplitBlockSize,
}
impl CustomProperty {
    const fn name(self) -> &'static str {
        match self {
            Self::ProgressValue => "--fc-progress-value",
            Self::ToolProgress => "--fc-tool-progress",
            Self::SliderPct => "--fc-slider-pct",
            Self::SplitSize => "--fc-split-size",
            Self::SplitBlockSize => "--fc-split-block-size",
        }
    }
}

/// A `style` attribute holding exactly one upstream custom property.
pub(crate) fn custom_property(property: CustomProperty, value: impl fmt::Display) -> Attribute {
    Attribute::new(
        "style",
        format!("{}: {value}", property.name()),
        None,
        false,
    )
}

/// The data grid's column template: an `auto` track for row headers when present, then
/// `columns` equal tracks.
///
/// The one inline style that is not a custom property. `grid.css` sizes implicit tracks
/// with `grid-auto-columns`, and rows are `display: contents`, so nothing creates the
/// tracks and every cell stacks in one column. Upstream's binding and its fixture both
/// write this declaration inline for that reason; it is layout the markup's shape
/// decides, not a colour or a theme value.
pub(crate) fn grid_columns(row_headers: bool, columns: usize) -> Attribute {
    let lead = if row_headers { "auto " } else { "" };
    Attribute::new(
        "style",
        format!("grid-template-columns: {lead}repeat({columns}, minmax(0, 1fr))"),
        None,
        false,
    )
}

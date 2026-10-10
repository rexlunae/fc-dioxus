//! Token vocabulary generated from the pinned future-css build.
//!
//! Regenerate with `cargo xtask sync-upstream`. Values are upstream data, never
//! authored here. High-contrast overrides remain in upstream's theme stylesheet.
#![forbid(unsafe_code)]

mod generated;
pub use generated::Token;

/// The schemes represented in upstream's token JSON.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum Scheme {
    /// Light scheme.
    #[default]
    Light,
    /// Dark scheme.
    Dark,
}

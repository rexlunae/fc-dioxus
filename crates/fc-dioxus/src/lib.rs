//! Typed components over the pinned future-css stylesheets.
//!
//! ```
//! use fc_dioxus::prelude::*;
//! fn app() -> Element {
//!     rsx! { FutureCss { Divider { inset: DividerInset::Middle } } }
//! }
//! ```
#![forbid(unsafe_code)]

pub mod behavior;
pub mod components;
pub mod core;
pub mod models;
pub mod prelude;
#[cfg(feature = "testing")]
pub mod testing;
pub mod theme;

pub use fc_tokens as tokens;

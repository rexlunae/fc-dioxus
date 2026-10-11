//! Shared typed contracts, classes, state, and host access.

/// Stylesheet handles generated from the pinned upstream build.
pub mod assets;
pub mod attrs;
pub mod class;
pub mod controlled;
/// Upstream's icon set, generated from its `dist/icons.json`.
pub mod icons;
pub mod id;
pub(crate) mod link;
pub mod measure;
pub mod platform;
pub mod state;
pub mod text;

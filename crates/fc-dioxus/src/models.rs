//! Pure Rust models from the same upstream revision as the stylesheets.
//!
//! JavaScript bindings and upstream's standalone WASM runtime are disabled.
//! Components share these models instead of duplicating selection or text logic.

/// Context menu navigation and typeahead.
pub use fc_context_menu as context_menu;
/// Shared model runtime and ABI version.
pub use fc_core as runtime;
/// Line and inline diff models.
pub use fc_diff as diff;
/// Git status and tree models.
pub use fc_git as git;
/// Data grid selection and navigation.
pub use fc_grid as grid;
/// Shared text models and offsets.
pub use fc_model as text;
/// Rich text operations.
pub use fc_rich_text as rich_text;
/// Select navigation, selection, and typeahead.
pub use fc_select as select;

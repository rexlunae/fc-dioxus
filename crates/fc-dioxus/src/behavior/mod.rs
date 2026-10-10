//! Keyboard and focus behaviour shared by several components. Pure decisions are plain
//! functions with unit tests; the hooks around them only read and write signals.

pub mod context;
pub mod date;
pub mod roving;
pub mod sort;
pub mod splitter;
pub mod url;

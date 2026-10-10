//! The initial rule catalogue. Every rule has failure and legitimate-pattern tests.

mod image_alt;
mod references;
mod tabindex;

use crate::Rule;
pub use image_alt::ImageAlt;
pub use references::{DuplicateId, IdRef};
pub use tabindex::Tabindex;

/// All currently implemented rules.
#[must_use]
pub fn catalogue() -> [&'static dyn Rule; 4] {
    [&ImageAlt, &IdRef, &DuplicateId, &Tabindex]
}

//! A single controlled/uncontrolled state convention for all components.

use dioxus::prelude::*;

/// A state handle that either owns a value or requests updates from its owner.
pub struct Controllable<T: Clone + PartialEq + 'static> {
    value: Option<ReadSignal<T>>,
    local: Signal<T>,
    on_change: Option<EventHandler<T>>,
}
// Every field is a copyable handle, whatever T is; a derive would demand T: Copy and
// make a String-valued handle unusable in two closures.
#[allow(clippy::expl_impl_clone_on_copy)]
impl<T: Clone + PartialEq + 'static> Clone for Controllable<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: Clone + PartialEq + 'static> Copy for Controllable<T> {}
impl<T: Clone + PartialEq + 'static> Controllable<T> {
    /// Read the current value, subscribing the rendering component to changes.
    #[must_use]
    pub fn get(&self) -> T {
        self.value
            .map_or_else(|| self.local.read().clone(), |value| value.read().clone())
    }
    /// Store and notify in uncontrolled mode; only notify in controlled mode.
    pub fn set(&self, next: T) {
        if self.get() == next {
            return;
        }
        if self.value.is_none() {
            let mut local = self.local;
            local.set(next.clone());
        }
        if let Some(handler) = self.on_change {
            handler.call(next);
        }
    }
}

/// Use controlled input when present, or initialize local state exactly once.
pub fn use_controllable<T: Clone + PartialEq + 'static>(
    value: Option<ReadSignal<T>>,
    default: impl FnOnce() -> T,
    on_change: Option<EventHandler<T>>,
) -> Controllable<T> {
    let local = use_signal(default);
    Controllable {
        value,
        local,
        on_change,
    }
}

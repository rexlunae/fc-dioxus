//! Roving focus: one tab stop for a set of items, arrows to move within it.
//!
//! A port of the arrow handling in upstream's `tabs.ts` and `toolbar.ts`: arrows follow
//! the writing direction (in RTL, Left moves forward), orientation decides which arrows
//! apply, Home and End jump to the ends, and the step wraps.

use crate::{core::state::Orientation, theme::Direction};
use dioxus::prelude::*;
use std::rc::Rc;

/// Where a key asks focus to go.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    /// The next item, wrapping.
    Next,
    /// The previous item, wrapping.
    Previous,
    /// The first item.
    First,
    /// The last item.
    Last,
}

/// The move a key makes, or `None` for keys the set leaves alone, such as Left and Right
/// in a vertical list, so the page can still scroll sideways.
#[must_use]
pub fn intent(key: &Key, orientation: Orientation, direction: Direction) -> Option<Move> {
    let rtl = direction == Direction::Rtl;
    match (key, orientation) {
        (Key::Home, _) => Some(Move::First),
        (Key::End, _) => Some(Move::Last),
        (Key::ArrowDown, Orientation::Vertical) => Some(Move::Next),
        (Key::ArrowUp, Orientation::Vertical) => Some(Move::Previous),
        (Key::ArrowRight, Orientation::Horizontal) => {
            Some(if rtl { Move::Previous } else { Move::Next })
        }
        (Key::ArrowLeft, Orientation::Horizontal) => {
            Some(if rtl { Move::Next } else { Move::Previous })
        }
        _ => None,
    }
}

/// The index focus moves to from `current`, skipping items `reachable` rejects.
/// `None` when nothing is reachable.
///
/// ```
/// use fc_dioxus::behavior::roving::{target, Move};
/// let enabled = [true, false, true, true];
/// let reachable = |i: usize| enabled.get(i).copied().unwrap_or(false);
/// assert_eq!(target(4, 0, Move::Next, reachable), Some(2));
/// assert_eq!(target(4, 3, Move::Next, reachable), Some(0));
/// assert_eq!(target(4, 0, Move::Previous, reachable), Some(3));
/// assert_eq!(target(4, 2, Move::First, reachable), Some(0));
/// assert_eq!(target(4, 0, Move::Last, reachable), Some(3));
/// assert_eq!(target(0, 0, Move::Next, reachable), None);
/// ```
#[must_use]
pub fn target(
    len: usize,
    current: usize,
    step: Move,
    reachable: impl Fn(usize) -> bool,
) -> Option<usize> {
    if len == 0 {
        return None;
    }
    match step {
        Move::First => (0..len).find(|&i| reachable(i)),
        Move::Last => (0..len).rev().find(|&i| reachable(i)),
        Move::Next => (1..=len)
            .map(|n| (current + n) % len)
            .find(|&i| reachable(i)),
        Move::Previous => (1..=len)
            .map(|n| (current + len - n % len) % len)
            .find(|&i| reachable(i)),
    }
}

/// One member of a roving set.
#[derive(Clone)]
pub(crate) struct Member {
    pub(crate) key: u64,
    pub(crate) value: String,
    /// Can take focus at all (not natively disabled).
    pub(crate) focusable: bool,
    /// Can be chosen (neither disabled nor unavailable).
    pub(crate) enabled: bool,
    pub(crate) mounted: Option<Rc<MountedData>>,
}

/// The members of a set in the order they first rendered, which is document order for
/// children written in place.
#[derive(Clone, Copy)]
pub(crate) struct Roster {
    members: Signal<Vec<Member>>,
    next: Signal<u64>,
}
impl Roster {
    pub(crate) fn new() -> Self {
        Self {
            members: Signal::new(Vec::new()),
            next: Signal::new(0),
        }
    }
    /// Snapshot without subscribing.
    pub(crate) fn peek(&self) -> Vec<Member> {
        self.members.peek().clone()
    }
    /// Snapshot, re-rendering the caller when membership changes.
    pub(crate) fn read(&self) -> Vec<Member> {
        self.members.read().clone()
    }
    fn join(mut self, value: String, focusable: bool, enabled: bool) -> u64 {
        let key = *self.next.peek();
        self.next.set(key + 1);
        self.members.write().push(Member {
            key,
            value,
            focusable,
            enabled,
            mounted: None,
        });
        key
    }
    fn update(mut self, key: u64, value: &str, focusable: bool, enabled: bool) {
        let stale = self.members.peek().iter().any(|m| {
            m.key == key && (m.value != value || m.focusable != focusable || m.enabled != enabled)
        });
        if stale {
            if let Some(member) = self.members.write().iter_mut().find(|m| m.key == key) {
                value.clone_into(&mut member.value);
                member.focusable = focusable;
                member.enabled = enabled;
            }
        }
    }
    pub(crate) fn mount(mut self, key: u64, mounted: Rc<MountedData>) {
        if let Some(member) = self.members.write().iter_mut().find(|m| m.key == key) {
            member.mounted = Some(mounted);
        }
    }
    fn leave(mut self, key: u64) {
        self.members.write().retain(|m| m.key != key);
    }
}

/// Join the nearest roster for as long as the calling component lives. Returns the
/// member's key, or `None` outside a roster.
pub(crate) fn use_member(value: &str, focusable: bool, enabled: bool) -> Option<(Roster, u64)> {
    let roster = try_consume_context::<Roster>();
    let joined = use_hook(|| roster.map(|r| (r, r.join(value.to_owned(), focusable, enabled))));
    if let Some((roster, key)) = joined {
        roster.update(key, value, focusable, enabled);
    }
    use_drop(move || {
        if let Some((roster, key)) = joined {
            roster.leave(key);
        }
    });
    joined
}

/// Move focus to a member after a key press.
pub(crate) fn focus(member: &Member) {
    if let Some(mounted) = member.mounted.clone() {
        spawn(async move {
            let _ = mounted.set_focus(true).await;
        });
    }
}

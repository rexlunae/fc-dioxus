//! The toast policy, mirroring upstream's toast.ts at the pin.
#![forbid(unsafe_code)]

use fc_dioxus::components::toast::{default_timeout, overflow};
use fc_dioxus::prelude::*;

#[test]
fn danger_never_expires_and_actions_get_half_as_long_again() {
    assert_eq!(default_timeout(ToastTone::Danger, false, 6000), 0);
    assert_eq!(default_timeout(ToastTone::Danger, true, 6000), 0);
    assert_eq!(default_timeout(ToastTone::Info, false, 6000), 6000);
    assert_eq!(default_timeout(ToastTone::Success, true, 6000), 9000);
    assert_eq!(
        default_timeout(ToastTone::Warning, true, u32::MAX),
        u32::MAX
    );
}

#[test]
fn overflow_drops_the_oldest_but_never_the_focused_toast() {
    assert_eq!(overflow(&[1, 2, 3], 3, None), Vec::<u64>::new());
    assert_eq!(overflow(&[1, 2, 3, 4], 3, None), vec![1]);
    assert_eq!(overflow(&[1, 2, 3, 4, 5], 3, Some(1)), vec![2]);
    assert_eq!(overflow(&[], 0, None), Vec::<u64>::new());
}

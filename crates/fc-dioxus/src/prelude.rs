//! The supported API for applications using `use fc_dioxus::prelude::*`.

pub use crate::fc_modifier;
pub use crate::{
    components::alert::{Alert, AlertIcon, AlertTone, Dismiss, Urgency},
    components::avatar::{Avatar, AvatarGroup, AvatarSize},
    components::badge::{Badge, BadgeTone, Kbd, Tag, TagRemove},
    components::breadcrumb::{Breadcrumb, Crumb},
    components::button::{Button, ButtonProps, ButtonSize, ButtonType, ButtonVariant},
    components::card::*,
    components::divider::{Divider, DividerInset, DividerProps},
    components::loading::{
        Loading, Progress, ProgressRing, ProgressValue, RingSize, Skeleton, SkeletonShape,
    },
    components::spinner::{
        ShimmerText, Spinner, SpinnerKind, SpinnerLabel, SpinnerSize, SpinnerTone,
    },
    core::{
        attrs::{FcData, Kind, Timestamp},
        class::{Bem, ClassList, Modifier},
        controlled::{use_controllable, Controllable},
        id::{use_id, Id},
        measure::Percent,
        state::{Availability, HeadingLevel, Lifecycle, MessageRole, Orientation},
        text::{AccessibleName, Alt},
    },
    theme::{Brand, Design, Direction, FutureCss, FutureCssProps, Palette, Stylesheets, Theme},
};
pub use dioxus::prelude::*;

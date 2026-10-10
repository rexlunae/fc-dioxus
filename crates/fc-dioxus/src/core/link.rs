//! Links that work with or without `dioxus-router`.
//!
//! Every component that renders a link goes through [`use_link`]. Without the `router`
//! feature, or outside a `Router`, the href is used as written and the browser follows
//! it. With the feature, inside a `Router`, an internal href is written with the
//! history's prefix, a plain left click navigates in-app instead of reloading, and a
//! link to the current route is marked current; modified clicks (new tab, new window)
//! and external links are left to the browser, as the router's own `Link` does.

use dioxus::prelude::*;

/// A link resolved against the router, if there is one.
#[derive(Clone, PartialEq)]
pub(crate) struct ResolvedLink {
    /// What to write in `href`.
    pub(crate) href: Option<String>,
    /// The route to push on a plain click, when the router owns this link.
    #[cfg_attr(not(feature = "router"), allow(dead_code))]
    route: Option<String>,
    /// The link points at the route being shown.
    pub(crate) current: bool,
}

impl ResolvedLink {
    /// Follow the link in-app when the router owns it and the click is a plain one.
    /// Returns whether the click was handled.
    #[cfg_attr(not(feature = "router"), allow(clippy::unused_self))]
    pub(crate) fn follow(&self, event: &MouseEvent) -> bool {
        #[cfg(feature = "router")]
        if let (Some(route), Some(router)) = (&self.route, dioxus::router::try_router()) {
            let plain = event.modifiers().is_empty()
                && event.trigger_button() == Some(dioxus::html::input_data::MouseButton::Primary);
            if plain {
                event.prevent_default();
                router.push(route.clone());
                return true;
            }
        }
        let _ = event;
        false
    }
}

/// Resolve an href against the nearest router, if any.
pub(crate) fn use_link(href: Option<String>) -> ResolvedLink {
    #[cfg(feature = "router")]
    if let (Some(path), Some(router)) = (&href, dioxus::router::try_router()) {
        // A same-page fragment stays a plain anchor.
        if !path.starts_with('#') {
            let target: dioxus::router::NavigationTarget = path.clone().into();
            if let dioxus::router::NavigationTarget::Internal(route) = target {
                let current = router.full_route_string() == route;
                return ResolvedLink {
                    href: Some(router.prefix().unwrap_or_default() + &route),
                    route: Some(route),
                    current,
                };
            }
        }
    }
    ResolvedLink {
        href,
        route: None,
        current: false,
    }
}

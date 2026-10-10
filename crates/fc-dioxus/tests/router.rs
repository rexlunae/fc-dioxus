//! Links inside a dioxus-router Router: prefixed hrefs and automatic current state.
#![forbid(unsafe_code)]
#![cfg(feature = "router")]

use fc_dioxus::prelude::*;
use std::rc::Rc;

#[derive(Clone, Routable, PartialEq, Debug)]
enum Route {
    #[route("/inbox")]
    Inbox {},
    #[route("/projects")]
    Projects {},
}

#[component]
fn Inbox() -> Element {
    rsx! {
        NavSidebar { label: name!("Main"), collapsible: false, title: rsx! { "App" },
            NavList {
                NavLink { label: "Inbox", href: "/inbox" }
                NavLink { label: "Projects", href: "/projects" }
            }
        }
        LinkButton { href: "/projects", "Projects" }
        LinkButton { href: "https://example.com/", "Elsewhere" }
        Crumb { href: "#section", "Anchor" }
    }
}

#[component]
fn Projects() -> Element {
    rsx! {}
}

#[component]
fn App() -> Element {
    use_hook(|| {
        dioxus::history::provide_history_context(Rc::new(
            dioxus::history::MemoryHistory::with_initial_path("/inbox"),
        ));
    });
    rsx! { Router::<Route> {} }
}

#[test]
fn the_link_to_the_current_route_is_current_and_others_are_not() {
    let html = dioxus_ssr::render_element(rsx! { App {} });
    assert!(
        html.contains(r#"href="/inbox" aria-current="page""#),
        "{html}"
    );
    assert!(html.contains(r#"href="/projects""#), "{html}");
    assert_eq!(html.matches(r#"aria-current="page""#).count(), 1, "{html}");
    // External and same-page links are left exactly as written.
    assert!(html.contains(r#"href="https://example.com/""#), "{html}");
    assert!(html.contains(r##"href="#section""##), "{html}");
}

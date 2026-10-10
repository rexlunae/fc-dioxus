use fc_dioxus::prelude::*;
use std::rc::Rc;

// example:start
/// Every link-like component, inside a `dioxus-router` `Router`: plain clicks navigate
/// in-app, the link to the route being shown is marked current, and modified clicks
/// still open a new tab. This example keeps its routes in memory so it does not change
/// the page's address.
#[component]
pub fn States() -> Element {
    // An app's Router uses the platform's history; this one is scoped to the example.
    use_hook(|| {
        dioxus::history::provide_history_context(Rc::new(
            dioxus::history::MemoryHistory::with_initial_path("/inbox"),
        ));
    });
    rsx! { Router::<Route> {} }
}

#[derive(Clone, Routable, PartialEq, Debug)]
#[rustfmt::skip]
enum Route {
    #[layout(Shell)]
        #[route("/inbox")]
        Inbox {},
        #[route("/projects")]
        Projects {},
        #[route("/projects/:id")]
        Project { id: u32 },
}

#[component]
fn Shell() -> Element {
    const SPLIT: Bem = Bem::new("fc-cluster");
    const STACK: Bem = Bem::new("fc-stack");
    let icon = || rsx! { NavIcon { path { d: "M3 4h14v9H8l-4 3v-3H3z" } } };
    rsx! {
        div { class: SPLIT.block(),
            NavSidebar { label: name!("Routed workspace"), collapsible: false, title: rsx! { "Workspace" },
                NavList {
                    NavLink { label: "Inbox", href: Route::Inbox {}.to_string(), icon: icon() }
                    NavLink { label: "Projects", href: Route::Projects {}.to_string(), icon: icon() }
                }
            }
            div { class: STACK.block(), Outlet::<Route> {} }
        }
    }
}

#[component]
fn Inbox() -> Element {
    rsx! {
        h3 { "Inbox" }
        p { "Nothing new." }
        LinkButton { href: Route::Projects {}.to_string(), variant: ButtonVariant::Primary, "Go to projects" }
    }
}

#[component]
fn Projects() -> Element {
    rsx! {
        h3 { "Projects" }
        List {
            for id in 1..=3_u32 {
                ListItem { key: "{id}", headline: rsx! { "Project {id}" }, action: ListAction::Link(Route::Project { id }.to_string()) }
            }
        }
    }
}

#[component]
fn Project(id: u32) -> Element {
    rsx! {
        Breadcrumb {
            Crumb { href: Route::Projects {}.to_string(), "Projects" }
            Crumb { "Project {id}" }
        }
        h3 { "Project {id}" }
        Pagination { label: name!("Projects"), page: id, pages: 3, href: move |n: u32| Route::Project { id: n }.to_string() }
        LinkButton { href: Route::Inbox {}.to_string(), variant: ButtonVariant::Secondary, "Back to the inbox" }
    }
}

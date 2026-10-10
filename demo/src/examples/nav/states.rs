use fc_dioxus::prelude::*;

// example:start
/// The sidebar expanded and as an icon rail. The toggle's state is what the stylesheet
/// draws; in the rail, each link's label becomes its title.
#[component]
pub fn States() -> Element {
    const GRID: Bem = Bem::new("fc-grid-auto");
    let reason = use_id();
    let icon = || rsx! { NavIcon { path { d: "M3 4h14v9H8l-4 3v-3H3z" } } };
    rsx! {
        div { class: GRID.block(),
            for (label, expanded) in [(name!("Workspace, expanded"), true), (name!("Workspace, rail"), false)] {
                NavSidebar { label, default_expanded: expanded, title: rsx! { "Workspace" },
                    NavHeading { "Chats" }
                    NavList {
                        NavLink { label: "Current page", href: "#nav", current: NavCurrent::Page, icon: icon(), meta: rsx! { "3" } }
                        NavLink { label: "Section of the current page", href: "#nav", current: NavCurrent::Section, icon: icon(),
                            nested: rsx! { NavList { nested: true, NavLink { label: "Nested", href: "#nav" } } } }
                        NavLink { label: "Rest", href: "#nav", icon: icon() }
                        NavLink { label: "Disabled", href: "#nav", icon: icon(), availability: Availability::Disabled }
                    }
                    NavGroup { label: rsx! { "Open group" }, open: true, NavLink { label: "Release notes", href: "#nav", icon: icon() } }
                    NavGroup { label: rsx! { "Closed group" }, NavLink { label: "Hidden", href: "#nav" } }
                    NavFooter { NavLink { label: "Settings", href: "#nav", icon: icon() } }
                }
            }
        }
        p { id: "{reason}", "Use the toggle beside the title to switch between the sidebar and the rail." }
    }
}

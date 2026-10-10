use fc_dioxus::prelude::*;

// example:start
/// Links to each level, with the current page last and unlinked.
#[component]
pub fn States() -> Element {
    rsx! {
        Breadcrumb {
            Crumb { href: "#breadcrumb", "Projects" }
            Crumb { href: "#breadcrumb", "A project name long enough to truncate somewhere" }
            Crumb { "Settings" }
        }
    }
}

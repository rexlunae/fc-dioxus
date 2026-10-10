use fc_dioxus::prelude::*;

// example:start
/// The current destination, a count badge, a dot badge, and an unavailable item. Each
/// badge is also said in words, since the drawn badge is hidden with its icon.
#[component]
pub fn States() -> Element {
    let mut current = use_signal(|| "chats");
    let reason = use_id();
    let names = ["Primary", "12 unread", "new activity"].map(AccessibleName::try_from);
    let [Ok(primary), Ok(unread), Ok(activity)] = names else {
        return rsx! {};
    };
    rsx! {
        Navbar { label: primary,
            NavbarItem { href: "#chats", current: current() == "chats", onclick: move |_| current.set("chats"),
                icon: rsx! { "💬" }, label: rsx! { "Chats" } }
            NavbarItem { href: "#inbox", current: current() == "inbox", onclick: move |_| current.set("inbox"),
                icon: rsx! { "📥" }, label: rsx! { "Inbox" }, badge: NavBadge::Count { count: 12, announce: unread } }
            NavbarItem { href: "#projects", current: current() == "projects", onclick: move |_| current.set("projects"),
                icon: rsx! { "📁" }, label: rsx! { "Projects" }, badge: NavBadge::Dot { announce: activity } }
            NavbarItem { href: "#settings", availability: Availability::Unavailable { reason: reason.clone() },
                icon: rsx! { "⚙" }, label: rsx! { "Settings and administration" } }
        }
        p { id: "{reason}", "Only administrators can open settings." }
    }
}

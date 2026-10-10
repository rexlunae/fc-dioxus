use fc_dioxus::prelude::*;

// A silhouette, inlined so the example needs no network or asset pipeline.
const PORTRAIT: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 40 40'%3E%3Crect width='40' height='40' fill='%23d2d4da'/%3E%3Ccircle cx='20' cy='16' r='7' fill='%2370747d'/%3E%3Cpath d='M6 40a14 14 0 0 1 28 0z' fill='%2370747d'/%3E%3C/svg%3E";

// example:start
/// Every size, square and neutral avatars, and a group with an overflow count.
#[component]
pub fn States() -> Element {
    const CLUSTER: Bem = Bem::new("fc-cluster");
    const STACK: Bem = Bem::new("fc-stack");
    rsx! {
        div { class: STACK.block(),
            div { class: CLUSTER.block(),
                // Each avatar sits beside a visible name here, so it is decoration.
                Avatar { name: "Ada Lovelace", alt: Alt::Decorative, size: AvatarSize::Xs }
                Avatar { name: "Ada Lovelace", alt: Alt::Decorative, size: AvatarSize::Sm }
                Avatar { name: "Ada Lovelace", alt: Alt::Decorative }
                Avatar { name: "Ada Lovelace", alt: Alt::Decorative, size: AvatarSize::Lg }
                Avatar { name: "Ada Lovelace", alt: Alt::Decorative, size: AvatarSize::Xl }
                span { "Ada Lovelace" }
            }
            div { class: CLUSTER.block(),
                Avatar { name: "Build bot", alt: Alt::Decorative, initials: "✦", square: true }
                Avatar { name: "Unknown user", alt: Alt::Decorative, initials: "?", neutral: true }
                AvatarGroup { overflow: 4, size: AvatarSize::Sm,
                    Avatar { name: "Ada", alt: Alt::Decorative, size: AvatarSize::Sm }
                    Avatar { name: "Babbage", alt: Alt::Decorative, size: AvatarSize::Sm }
                }
                span { "Ada, Babbage, and 4 others reviewed this." }
            }
            div { class: CLUSTER.block(),
                // Standing alone, the avatar is announced by name.
                Avatar { name: "Grace Hopper", size: AvatarSize::Lg }
                // A picture takes the name as its alternative text.
                Avatar { name: "Default profile", size: AvatarSize::Lg, src: PORTRAIT }
            }
        }
    }
}

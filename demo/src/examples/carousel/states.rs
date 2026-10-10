use fc_dioxus::prelude::*;

// example:start
/// A row of templates that pages by most of its width, a hero carousel, and a free
/// scrolling row. Each scrolls by touch, trackpad, and keyboard without script.
#[component]
pub fn States() -> Element {
    const STACK: Bem = Bem::new("fc-stack");
    let templates = [
        "Weekly report",
        "Meeting notes with a long title that wraps",
        "Code review",
        "Release plan",
    ];
    rsx! {
        div { class: STACK.block(),
            Carousel { label: name!("Templates"),
                // The first slide opens its template; captions can hold links.
                Slide { index: 1, total: 4, caption: rsx! { a { href: "#carousel", "{templates[0]}" } } }
                for (index, title) in (1..).zip(templates).skip(1) {
                    Slide { key: "{index}", index, total: 4, caption: rsx! { "{title}" } }
                }
            }
            Carousel { label: name!("Featured"), layout: CarouselLayout::Hero,
                Slide { index: 1, total: 2, caption: rsx! { "Featured: agents that write tests" } }
                Slide { index: 2, total: 2, caption: rsx! { "New: the context strip" } }
            }
            Carousel { label: name!("Recent files"), layout: CarouselLayout::Free,
                for index in 1..=6_u32 {
                    Slide { key: "{index}", index, total: 6, caption: rsx! { "File {index}" } }
                }
            }
        }
    }
}

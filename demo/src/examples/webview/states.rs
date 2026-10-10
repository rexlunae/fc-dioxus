use fc_dioxus::prelude::*;

const PAGES: [(&str, &str); 2] = [
    ("Docs", "https://example.com/"),
    ("A tab whose title is long", "https://example.org/"),
];

// example:start
/// Tabs that switch pages, a page served without TLS, and an address the allowlist
/// refused, with its retry overlay. Type an address and press Enter; Back and Forward
/// walk this control's own history.
#[component]
pub fn States() -> Element {
    const STACK: Bem = Bem::new("fc-stack");
    let mut tab = use_signal(|| 0_usize);
    let mut log = use_signal(String::new);
    let src = PAGES
        .get(tab())
        .map_or("about:blank", |(_, url)| url)
        .to_owned();
    rsx! {
        div { class: STACK.block(),
            Webview {
                title: name!("Documentation preview"),
                src,
                tabs: PAGES.iter().map(|(label, _)| (*label).to_owned()).collect::<Vec<_>>(),
                selected_tab: tab(),
                ontab: move |index| tab.set(index),
                onnavigate: move |url| log.set(format!("Navigated to {url}")),
                onstate: move |state| if state == WebviewState::Error { log.set("Refused or failed to load".to_owned()) },
            }
            p { role: "status", "{log}" }
            Webview { title: name!("Plain HTTP page"), src: "http://example.com/" }
            Webview { title: name!("Refused page"), src: "javascript:alert(1)" }
        }
    }
}

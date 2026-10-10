use fc_dioxus::prelude::*;

// example:start
/// Paging within the app, plus crawlable links on the last page with chosen items.
#[component]
pub fn States() -> Element {
    const STACK: Bem = Bem::new("fc-stack");
    let mut page = use_signal(|| 1);
    let names = ["Pagination, first page", "Pagination, last page"].map(AccessibleName::try_from);
    let [Ok(first), Ok(last)] = names else {
        return rsx! {};
    };
    rsx! {
        div { class: STACK.block(),
            // Page windows come from page_window: first, last, neighbours, and gaps.
            Pagination { label: first, page: page(), pages: 12, onchange: move |next| page.set(next) }
            p { role: "status", "Showing page {page} of 12." }
            Pagination { label: last, page: 12, pages: 12,
                items: vec![PageItem::Page(11), PageItem::Page(12)],
                href: move |number: u32| format!("#pagination-{number}") }
        }
    }
}

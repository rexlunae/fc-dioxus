//! Numbered page navigation, from pagination.css.
use crate::{
    core::{assets, attrs::component_attributes},
    prelude::*,
    theme::use_stylesheets,
};
const BEM: Bem = Bem::new("fc-pagination");
const LINK: Bem = Bem::new("fc-pagination__link");
fc_modifier! { enum PageDirection for LINK { Page => None, Prev => "prev", Next => "next" } }

/// One entry in the page list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PageItem {
    /// A numbered page, counted from 1.
    Page(u32),
    /// Skipped pages, drawn as an ellipsis.
    Gap,
}

/// The pages to show: the first and last, `siblings` either side of `page`, and a gap
/// wherever pages are skipped. A gap that would hide a single page shows that page
/// instead, because an ellipsis standing for one number saves nothing.
///
/// ```
/// use fc_dioxus::components::pagination::{page_window, PageItem::{Gap, Page}};
/// assert_eq!(page_window(1, 12, 1), [Page(1), Page(2), Gap, Page(12)]);
/// assert_eq!(page_window(6, 12, 1), [Page(1), Gap, Page(5), Page(6), Page(7), Gap, Page(12)]);
/// assert_eq!(page_window(3, 12, 1), [Page(1), Page(2), Page(3), Page(4), Gap, Page(12)]);
/// assert_eq!(page_window(1, 0, 1), []);
/// ```
#[must_use]
pub fn page_window(page: u32, pages: u32, siblings: u32) -> Vec<PageItem> {
    if pages == 0 {
        return Vec::new();
    }
    let page = page.clamp(1, pages);
    let low = page.saturating_sub(siblings).max(1);
    let high = page.saturating_add(siblings).min(pages);
    let mut shown: Vec<u32> = vec![1];
    shown.extend(low..=high);
    shown.push(pages);
    shown.sort_unstable();
    shown.dedup();
    let mut items = Vec::with_capacity(shown.len() + 2);
    let mut previous = 0;
    for number in shown {
        match number - previous {
            1 => {}
            2 => items.push(PageItem::Page(previous + 1)),
            _ => items.push(PageItem::Gap),
        }
        items.push(PageItem::Page(number));
        previous = number;
    }
    items
}

/// Previous and next links around a list of pages.
///
/// Give `href` to render crawlable links, or `onchange` to page within the app. The
/// items default to [`page_window`]; pass `items` to choose them yourself.
#[component]
pub fn Pagination(
    label: AccessibleName,
    page: u32,
    pages: u32,
    #[props(default = 1)] siblings: u32,
    #[props(default)] items: Option<Vec<PageItem>>,
    #[props(default)] href: Option<Callback<u32, String>>,
    #[props(default)] onchange: Option<EventHandler<u32>>,
    #[props(default = "Previous".to_owned())] previous: String,
    #[props(default = "Next".to_owned())] next: String,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let attributes = component_attributes(
        attributes,
        ClassList::new(BEM.block()).extend(class.as_deref()),
        &["aria-label"],
    );
    let items = items.unwrap_or_else(|| page_window(page, pages, siblings));
    let link =
        move |target: u32, direction: PageDirection, enabled: bool, current: bool, text: String| {
            let class = ClassList::new(LINK.block())
                .with_modifier(LINK, direction)
                .to_string();
            let availability = if enabled {
                Availability::Enabled
            } else {
                Availability::Disabled
            };
            let url = href.map(|href| href.call(target));
            let onclick = onchange.map(|onchange| {
                EventHandler::new(move |event: MouseEvent| {
                    // In-app paging replaces navigation; a link with an href still works
                    // when opened in a new tab.
                    event.prevent_default();
                    onchange.call(target);
                })
            });
            let url = url.or_else(|| onclick.is_some().then(|| format!("#page-{target}")));
            super::nav_target(
                class,
                url,
                onclick,
                current,
                &availability,
                Vec::new(),
                rsx! { "{text}" },
            )
        };
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_PAGINATION_CSS } }
        nav { aria_label: label.to_string(), ..attributes,
            ul { class: BEM.element("list"),
                li { {link(page.saturating_sub(1), PageDirection::Prev, page > 1, false, previous)} }
                for (index, item) in items.into_iter().enumerate() {
                    li { key: "{index}",
                        match item {
                            PageItem::Page(number) => link(number, PageDirection::Page, true, number == page, number.to_string()),
                            PageItem::Gap => rsx! { span { class: BEM.element("gap"), aria_hidden: "true", "…" } },
                        }
                    }
                }
                li { {link(page.saturating_add(1), PageDirection::Next, page < pages, false, next)} }
            }
        }
    }
}

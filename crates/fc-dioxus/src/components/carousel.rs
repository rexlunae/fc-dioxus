//! A scrolling row of slides with paging buttons, from carousel.css. A port of
//! upstream's `carousel.ts`.
use crate::{
    core::{
        assets,
        attrs::{carousel_control_attribute, component_attributes},
        platform::DomBridge,
    },
    prelude::*,
    theme::use_stylesheets,
};
const BEM: Bem = Bem::new("fc-carousel");
const BTN: Bem = Bem::new("fc-btn");
fc_modifier! {
    /// Carousel layouts.
    pub enum CarouselLayout for BEM {
        /// Slides snap, several to a view.
        Row => None,
        /// One large slide at a time.
        Hero => "hero",
        /// Free scrolling without snapping.
        Free => "free",
    }
}

/// How much of the visible width one press of a paging button moves: most of it, so the
/// last visible slide becomes the first and the reader keeps their place.
pub const PAGE: f64 = 0.85;

/// Whether a track is at its start and at its end, from upstream's `edges`. The offset
/// is taken as an absolute value because `scrollLeft` runs negative in RTL.
///
/// ```
/// use fc_dioxus::components::carousel::edges;
/// assert_eq!(edges(0.0, 1000.0, 400.0), (true, false));
/// assert_eq!(edges(-600.0, 1000.0, 400.0), (false, true));
/// assert_eq!(edges(0.0, 400.0, 400.0), (true, true));
/// ```
#[must_use]
pub fn edges(offset: f64, scroll_width: f64, client_width: f64) -> (bool, bool) {
    let offset = offset.abs();
    let max = scroll_width - client_width;
    (offset <= 1.0, offset >= max - 1.0)
}

/// A named region of slides that scrolls by touch, trackpad, and keyboard with no script;
/// the previous and next buttons page it for everyone else and are `aria-disabled` at
/// their end.
#[component]
pub fn Carousel(
    label: AccessibleName,
    #[props(default)] layout: CarouselLayout,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    /// [`Slide`]s.
    children: Element,
) -> Element {
    let id = use_id();
    // At the start before anything scrolls; whether there is an end is measured.
    let mut at = use_signal(|| (true, None::<bool>));
    let attributes = component_attributes(
        attributes,
        ClassList::new(BEM.block())
            .with_modifier(BEM, layout)
            .extend(class.as_deref()),
        &["aria-roledescription", "aria-label"],
    );
    let track_id = id.clone();
    let sync = move || {
        let id = track_id.clone();
        async move {
            if let Ok((start, end)) = DomBridge.scroll_edges(&id).await {
                at.set((start, Some(end)));
            }
        }
    };
    let (start, end) = at();
    let label_text = label.to_string();
    rsx! {
        if use_stylesheets() {
            document::Stylesheet { href: assets::COMPONENTS_BUTTON_CSS }
            document::Stylesheet { href: assets::COMPONENTS_CAROUSEL_CSS }
        }
        section { aria_roledescription: "carousel", aria_label: "{label_text}", ..attributes,
            div { class: BEM.element("controls"),
                button { class: control_class(), r#type: "button", aria_label: "Previous",
                    aria_disabled: start.then_some("true"),
                    onclick: { let id = id.clone(); move |_| { let id = id.clone(); async move { if !start { let _ = DomBridge.page(&id, false).await; } } } },
                    ..vec![carousel_control_attribute(false)], "‹"
                }
                button { class: control_class(), r#type: "button", aria_label: "Next",
                    aria_disabled: end.map(|end| end.to_string()),
                    onclick: { let id = id.clone(); move |_| { let id = id.clone(); async move { if end != Some(true) { let _ = DomBridge.page(&id, true).await; } } } },
                    ..vec![carousel_control_attribute(true)], "›"
                }
            }
            ul { class: BEM.element("track"), id: "{id}", tabindex: "0", aria_label: "{label_text}",
                onmounted: { let sync = sync.clone(); move |_| sync() },
                // Stylesheets and images change whether there is anywhere to scroll at all.
                onresize: { let sync = sync.clone(); move |_| sync() },
                onscroll: move |_| sync(),
                {children}
            }
        }
    }
}

fn control_class() -> String {
    ClassList::new(BTN.block())
        .with(BTN.modifier_raw("secondary"))
        .with(BTN.modifier_raw("icon"))
        .with(BTN.modifier_raw("sm"))
        .to_string()
}

/// One slide, announced as "n of total".
#[component]
pub fn Slide(
    index: u32,
    total: u32,
    #[props(default)] caption: Option<Element>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    rsx! {
        li { class: BEM.element("item"), aria_roledescription: "slide", aria_label: "{index} of {total}", ..attributes,
            {children}
            if let Some(caption) = caption { p { class: BEM.element("caption"), {caption} } }
        }
    }
}

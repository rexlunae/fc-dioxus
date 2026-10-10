//! Embedded browser chrome from webview.css: page tabs, back and forward, an address
//! bar with a security indicator, the frame, and a load-error overlay.
//!
//! Chrome, not a browser: it never reads or mediates the embedded page, which a
//! cross-origin frame forbids anyway. History is the control's own, since a frame's
//! cross-origin history is out of reach.
use crate::{
    behavior::{
        roving,
        url::{frame_url, is_secure},
    },
    core::{
        assets,
        attrs::{component_attributes, named_state_attribute},
        state::Orientation,
    },
    prelude::*,
    theme::{use_direction, use_stylesheets},
};
use std::rc::Rc;
const BEM: Bem = Bem::new("fc-webview");
const VISUALLY_HIDDEN: Bem = Bem::new("fc-u-visually-hidden");

/// Where the frame is in loading its page.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum WebviewState {
    /// Nothing requested yet.
    #[default]
    Idle,
    /// A navigation is in flight.
    Loading,
    /// The frame finished a navigation. Not proof of success: a 404 loads too.
    Loaded,
    /// The address was refused or the frame failed.
    Error,
}
impl WebviewState {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Loading => "loading",
            Self::Loaded => "loaded",
            Self::Error => "error",
        }
    }
}

/// Embedded browser chrome around a sandboxed frame. Changing `src` navigates; so does
/// the address bar, after [`frame_url`] refuses unsafe schemes. With `tabs`, a tab list
/// sits above the toolbar and reports choices through `ontab`.
#[component]
pub fn Webview(
    /// The frame's title, which names it for assistive technology.
    title: AccessibleName,
    /// The page to show; a change navigates.
    src: String,
    /// Page tab labels.
    #[props(default)]
    tabs: Vec<String>,
    #[props(default)] selected_tab: usize,
    #[props(default)] ontab: Option<EventHandler<usize>>,
    /// The frame's sandbox. Without `allow-same-origin` the page runs in an opaque
    /// origin and cannot reach this page's cookies or storage; add it knowingly.
    #[props(default = "allow-scripts allow-forms allow-popups".to_owned())]
    sandbox: String,
    /// Replaces the overlay's message after a failed load.
    #[props(default)]
    error: Option<String>,
    #[props(default)] onnavigate: Option<EventHandler<String>>,
    #[props(default)] onstate: Option<EventHandler<WebviewState>>,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    // The first page is part of the first render, so server rendering and fixtures show
    // it; later changes to `src` navigate through the effect below.
    let first = frame_url(&src);
    let mut history = use_signal(|| first.clone().map(|url| vec![url]).unwrap_or_default());
    let mut at = use_signal(|| 0_usize);
    let mut state = use_signal(|| {
        if first.is_ok() {
            WebviewState::Loading
        } else {
            WebviewState::Error
        }
    });
    let mut draft = use_signal(|| first.clone().unwrap_or_else(|_| src.clone()));
    let mut seen = use_signal(|| src.clone());
    let mut tab_nodes = use_signal(Vec::<(usize, Rc<MountedData>)>::new);
    let direction = use_direction();

    let mut set_state = move |next: WebviewState| {
        state.set(next);
        if let Some(onstate) = onstate {
            onstate.call(next);
        }
    };
    let mut go = move |raw: String, record: bool| match frame_url(&raw) {
        Ok(url) => {
            if record {
                // A new navigation discards the forward tail, as browser history does.
                let keep = if history.peek().is_empty() {
                    0
                } else {
                    at() + 1
                };
                let mut list = history.write();
                list.truncate(keep);
                list.push(url.clone());
                at.set(list.len() - 1);
            }
            draft.set(url.clone());
            set_state(WebviewState::Loading);
            if let Some(onnavigate) = onnavigate {
                onnavigate.call(url);
            }
        }
        // Refused rather than dropped: whoever typed it should learn it did not load.
        Err(_) => set_state(WebviewState::Error),
    };
    use_effect(use_reactive!(|src| {
        if *seen.peek() != src {
            seen.set(src.clone());
            go(src, true);
        }
    }));

    let current = history.read().get(at()).cloned().unwrap_or_default();
    let can_back = at() > 0;
    let can_forward = at() + 1 < history.read().len();
    let mut step = move |forward: bool| {
        let target = if forward {
            at() + 1
        } else {
            at().saturating_sub(1)
        };
        let url = history.peek().get(target).cloned();
        if let Some(url) = url {
            at.set(target);
            go(url, false);
        }
    };

    let mut attributes = component_attributes(
        attributes,
        ClassList::new(BEM.block()).extend(class.as_deref()),
        &[],
    );
    attributes.push(named_state_attribute(state().as_str()));
    let secure = is_secure(&current);
    let failed = state() == WebviewState::Error;
    let tab_count = tabs.len();
    let nav_class = BEM.element("nav");
    let back_class = ClassList::new(&nav_class)
        .with(BEM.element("nav--back"))
        .to_string();

    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_WEBVIEW_CSS } }
        div { ..attributes,
            if !tabs.is_empty() {
                div { class: BEM.element("tabs"), role: "tablist", aria_label: "Pages",
                    for (index, tab) in tabs.iter().enumerate() {
                        button { key: "{index}", r#type: "button", class: BEM.element("tab"), role: "tab",
                            aria_selected: (index == selected_tab).to_string(),
                            tabindex: if index == selected_tab { "0" } else { "-1" },
                            onmounted: move |event: MountedEvent| {
                                let mut nodes = tab_nodes.write();
                                nodes.retain(|(i, _)| *i != index);
                                nodes.push((index, event.data()));
                            },
                            onclick: move |_| if let Some(ontab) = ontab { ontab.call(index) },
                            onkeydown: move |event: KeyboardEvent| {
                                let Some(step) = roving::intent(&event.key(), Orientation::Horizontal, direction) else { return };
                                let Some(next) = roving::target(tab_count, index, step, |_| true) else { return };
                                event.prevent_default();
                                // Automatic activation: the page follows the focused tab.
                                if let Some(ontab) = ontab { ontab.call(next); }
                                let node = tab_nodes.peek().iter().find(|(i, _)| *i == next).map(|(_, n)| n.clone());
                                if let Some(node) = node {
                                    spawn(async move { let _ = node.set_focus(true).await; });
                                }
                            },
                            span { class: BEM.element("tab-label"), "{tab}" }
                        }
                    }
                }
            }
            div { class: BEM.element("toolbar"),
                // Disabled rather than hidden, so the toolbar never shifts under the pointer.
                Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, icon_label: name!("Back"), class: back_class,
                    availability: if can_back { Availability::Enabled } else { Availability::Disabled },
                    onclick: move |_| step(false), "\u{2190}" }
                Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, icon_label: name!("Forward"), class: nav_class.clone(),
                    availability: if can_forward { Availability::Enabled } else { Availability::Disabled },
                    onclick: move |_| step(true), "\u{2192}" }
                Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, icon_label: name!("Reload"), class: nav_class,
                    availability: if current.is_empty() { Availability::Disabled } else { Availability::Enabled },
                    onclick: { let current = current.clone(); move |_| go(current.clone(), false) }, "\u{21bb}" }
                // A real form, so Enter submits and mobile keyboards offer "Go".
                form { class: BEM.element("address"), role: "search",
                    onsubmit: move |event: FormEvent| {
                        event.prevent_default();
                        go(draft(), true);
                    },
                    match secure {
                        Some(true) => rsx! {
                            span { class: BEM.element("secure"),
                                span { aria_hidden: "true", "\u{1f512}" }
                                span { class: VISUALLY_HIDDEN.block(), "Secure" }
                            }
                        },
                        // The warning is spoken and shown: colour alone would hide it.
                        Some(false) => rsx! {
                            span { class: BEM.element("secure"), ..crate::spread![named_state_attribute("insecure")],
                                span { aria_hidden: "true", "\u{26a0}" }
                                span { "Not secure" }
                            }
                        },
                        None => rsx! {},
                    }
                    input { class: BEM.element("address-input"), r#type: "text", aria_label: "Address",
                        autocomplete: "off", spellcheck: "false", value: "{draft}",
                        oninput: move |event| draft.set(event.value()) }
                }
            }
            iframe { class: BEM.element("frame"), title: "{title}", src: "{current}", "sandbox": "{sandbox}",
                // The host's URL is not the embedded page's business.
                referrerpolicy: "strict-origin-when-cross-origin",
                // No camera, microphone, or location unless the host grants them.
                allow: "",
                onload: move |_| if !current_failed(state) { set_state(WebviewState::Loaded) },
            }
            div { class: BEM.element("overlay"), role: "alert", hidden: !failed,
                if failed {
                    p { class: BEM.element("error"), {error.clone().unwrap_or_else(|| "This page could not be loaded.".to_owned())} }
                    Button { variant: ButtonVariant::Secondary, size: ButtonSize::Sm,
                        onclick: move |_| {
                            let url = history.peek().get(at()).cloned();
                            if let Some(url) = url { go(url, false); }
                        },
                        "Retry"
                    }
                }
            }
        }
    }
}

/// A refused address leaves the old page in the frame, and its load must not clear the
/// error the refusal raised.
fn current_failed(state: Signal<WebviewState>) -> bool {
    *state.peek() == WebviewState::Error
}

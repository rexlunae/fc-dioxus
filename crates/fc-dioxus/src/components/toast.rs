//! Brief, non-blocking notifications and the queue that times them, from toast.css.
//!
//! The policy follows upstream's `toast.ts`: two live regions created up front (polite,
//! and assertive for danger), danger toasts that never expire on their own, a longer
//! default when there is an action, timers that pause while the pointer or focus is in
//! the stack, a cap past which the oldest unfocused toast goes, and toasts that never
//! take focus.
use crate::{
    core::{
        assets,
        attrs::{component_attributes, leaving_attribute, FcData, Kind},
        platform::DomBridge,
    },
    prelude::*,
    theme::use_stylesheets,
};
const TOAST: Bem = Bem::new("fc-toast");
const TOASTER: Bem = Bem::new("fc-toaster");
fc_modifier! {
    /// Toast tones.
    pub enum ToastTone for TOAST {
        /// No tone of its own. Toasts default to `Info` instead.
        Plain => None,
        /// Information.
        Info => "info",
        /// Something succeeded.
        Success => "success",
        /// Something needs attention soon.
        Warning => "warning",
        /// Something failed. Never expires on its own, and is announced assertively.
        Danger => "danger",
    }
}
impl ToastTone {
    const fn glyph(self) -> &'static str {
        match self {
            Self::Plain | Self::Info => "i",
            Self::Success => "✓",
            Self::Warning => "!",
            Self::Danger => "✕",
        }
    }
}
fc_modifier! { enum ToasterPlacement for TOASTER { Corner => None, Top => "top" } }

/// One optional action inside a toast, e.g. "Undo".
#[derive(Debug, Clone, PartialEq)]
pub struct ToastAction {
    /// Visible label.
    pub label: String,
    /// Called on activation; the toast is then dismissed.
    pub onaction: EventHandler<()>,
}

/// One toast. Use it directly for a static list, or let a [`Toaster`] render it.
#[component]
pub fn Toast(
    #[props(default = ToastTone::Info)] tone: ToastTone,
    #[props(default)] title: Option<String>,
    #[props(default)] body: Option<String>,
    #[props(default)] action: Option<ToastAction>,
    ondismiss: EventHandler<()>,
    /// The runtime id for `data-fc-id`.
    #[props(default)]
    fc_id: Option<Id>,
    #[props(default)] leaving: bool,
    /// Called with `true` when focus enters the toast and `false` when it leaves, so a
    /// queue can keep the toast someone is using.
    #[props(default)]
    onfocuschange: Option<EventHandler<bool>>,
) -> Element {
    let class = ClassList::new(TOAST.block())
        .with_modifier(TOAST, tone)
        .to_string();
    let mut data = FcData {
        kind: Some(Kind::Toast),
        id: fc_id,
        ..FcData::default()
    }
    .attributes();
    data.extend(leaving_attribute(leaving));
    // The name says what is dismissed, so several close buttons are not all "Dismiss".
    let what = title
        .clone()
        .or_else(|| body.clone())
        .unwrap_or_else(|| "notification".to_owned());
    rsx! {
        li { class,
            onfocusin: move |_| if let Some(on) = onfocuschange { on.call(true) },
            onfocusout: move |_| if let Some(on) = onfocuschange { on.call(false) },
            onkeydown: move |event| if event.key() == Key::Escape { event.prevent_default(); ondismiss.call(()); },
            ..data,
            span { class: TOAST.element("icon"), aria_hidden: "true", "{tone.glyph()}" }
            div { class: TOAST.element("content"),
                if let Some(title) = title { p { class: TOAST.element("title"), "{title}" } }
                if let Some(body) = body { p { class: TOAST.element("body"), "{body}" } }
            }
            if let Some(action) = action {
                button { r#type: "button", class: TOAST.element("action"),
                    onclick: move |_| { action.onaction.call(()); ondismiss.call(()); }, "{action.label}" }
            }
            button { r#type: "button", class: TOAST.element("close"), aria_label: "Dismiss: {what}",
                onclick: move |_| ondismiss.call(()), "✕" }
        }
    }
}

/// A static list of [`Toast`]s, e.g. for documentation. Live toasts belong in a
/// [`Toaster`].
#[component]
pub fn ToastList(children: Element) -> Element {
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_TOAST_CSS } }
        ol { class: TOASTER.element("list"), {children} }
    }
}

/// What to show in a toast.
#[derive(Debug, Clone, PartialEq)]
pub struct ToastOptions {
    /// Tone; info by default.
    pub tone: ToastTone,
    /// Short title.
    pub title: Option<String>,
    /// One sentence.
    pub body: Option<String>,
    /// Milliseconds before it goes; `Some(0)` keeps it. Defaults by [`default_timeout`].
    pub timeout: Option<u32>,
    /// One optional action.
    pub action: Option<ToastAction>,
}

impl Default for ToastOptions {
    fn default() -> Self {
        Self {
            tone: ToastTone::Info,
            title: None,
            body: None,
            timeout: None,
            action: None,
        }
    }
}

/// How long a toast stays when the caller does not say: never for danger, half as long
/// again with an action (deciding takes longer than reading), `base` otherwise.
#[must_use]
pub fn default_timeout(tone: ToastTone, has_action: bool, base: u32) -> u32 {
    if tone == ToastTone::Danger {
        0
    } else if has_action {
        base.saturating_add(base / 2)
    } else {
        base
    }
}

/// Which toasts to dismiss to get back to `max`: the oldest first, but never the one
/// holding focus. `keys` is in arrival order.
#[must_use]
pub fn overflow(keys: &[u64], max: usize, focused: Option<u64>) -> Vec<u64> {
    let excess = keys.len().saturating_sub(max);
    // As upstream's toast.ts: the oldest past the cap go, skipping a focused one rather
    // than evicting another in its place.
    keys.iter()
        .take(excess)
        .copied()
        .filter(|key| Some(*key) != focused)
        .collect()
}

#[derive(Debug, Clone, PartialEq)]
struct Live {
    key: u64,
    options: ToastOptions,
    remaining: u32,
    leaving: bool,
}

/// A handle for showing and dismissing toasts, from [`use_toasts`].
#[derive(Clone, Copy)]
pub struct Toasts {
    live: Signal<Vec<Live>>,
    next: Signal<u64>,
    focused: Signal<Option<u64>>,
    max: usize,
    base: u32,
}
impl Toasts {
    /// Show a toast; returns its key for [`Toasts::dismiss`].
    // Most callers fire and forget; the key is only for dismissing early.
    #[allow(clippy::must_use_candidate)]
    pub fn show(mut self, options: ToastOptions) -> u64 {
        let key = *self.next.peek();
        self.next.set(key + 1);
        let remaining = options
            .timeout
            .unwrap_or_else(|| default_timeout(options.tone, options.action.is_some(), self.base));
        self.live.write().push(Live {
            key,
            options,
            remaining,
            leaving: false,
        });
        let keys: Vec<u64> = self
            .live
            .peek()
            .iter()
            .filter(|t| !t.leaving)
            .map(|t| t.key)
            .collect();
        for old in overflow(&keys, self.max, *self.focused.peek()) {
            self.dismiss(old);
        }
        key
    }
    /// Dismiss a toast, letting it animate out first where the host can time it.
    pub fn dismiss(self, key: u64) {
        let mut live = self.live;
        if let Some(toast) = live.write().iter_mut().find(|t| t.key == key) {
            toast.leaving = true;
        }
        spawn(async move {
            // Reduced motion and hosts without timers remove at once.
            let _ = DomBridge.delay(400).await;
            live.write().retain(|t| t.key != key);
        });
    }
}

/// The toast queue provided by the nearest [`Toaster`].
///
/// # Panics
/// Never; outside a `Toaster` it returns a detached queue whose toasts render nowhere.
#[must_use]
pub fn use_toasts() -> Toasts {
    use_hook(|| {
        try_consume_context::<Toasts>().unwrap_or_else(|| Toasts {
            live: Signal::new(Vec::new()),
            next: Signal::new(0),
            focused: Signal::new(None),
            max: 3,
            base: 6000,
        })
    })
}

/// Provides [`use_toasts`] to its children and renders the toast region after them.
#[component]
pub fn Toaster(
    /// Names the region; "Notifications" when omitted.
    #[props(default)]
    label: Option<AccessibleName>,
    /// Centre at the top, for layouts whose block-end corner holds a composer.
    #[props(default)]
    top: bool,
    #[props(default = 3)] max: usize,
    /// Default lifetime in milliseconds.
    #[props(default = 6000)]
    timeout: u32,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let live = use_signal(Vec::<Live>::new);
    let next = use_signal(|| 0);
    let mut focused = use_signal(|| None::<u64>);
    let mut paused = use_signal(|| false);
    let toasts = use_context_provider(|| Toasts {
        live,
        next,
        focused,
        max,
        base: timeout,
    });
    // One ticker counts every toast down while the stack is not paused.
    use_future(move || async move {
        const TICK: u32 = 250;
        while DomBridge.delay(TICK).await.is_ok() {
            if *paused.peek() {
                continue;
            }
            let mut expired = Vec::new();
            let mut list = live;
            for toast in list
                .write()
                .iter_mut()
                .filter(|t| !t.leaving && t.remaining > 0)
            {
                toast.remaining = toast.remaining.saturating_sub(TICK);
                if toast.remaining == 0 {
                    expired.push(toast.key);
                }
            }
            for key in expired {
                toasts.dismiss(key);
            }
        }
    });
    let attributes = component_attributes(
        attributes,
        ClassList::new(TOASTER.block())
            .with_modifier(
                TOASTER,
                if top {
                    ToasterPlacement::Top
                } else {
                    ToasterPlacement::Corner
                },
            )
            .extend(class.as_deref()),
        &["aria-label"],
    );
    let label = label.map_or_else(|| "Notifications".to_owned(), |l| l.to_string());
    let render = move |danger: bool| {
        let items: Vec<Live> = live
            .read()
            .iter()
            .filter(|t| (t.options.tone == ToastTone::Danger) == danger)
            .cloned()
            .collect();
        rsx! {
            for toast in items {
                Toast { key: "{toast.key}", tone: toast.options.tone, title: toast.options.title.clone(),
                    body: toast.options.body.clone(), action: toast.options.action.clone(), leaving: toast.leaving,
                    fc_id: Id::try_from(format!("toast-{}", toast.key)).ok(),
                    ondismiss: move |()| toasts.dismiss(toast.key),
                    // The queue's cap skips this key while it holds focus.
                    onfocuschange: move |inside: bool| {
                        if inside { focused.set(Some(toast.key)); }
                        else if *focused.peek() == Some(toast.key) { focused.set(None); }
                    } }
            }
        }
    };
    rsx! {
        {children}
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_TOAST_CSS } }
        section { aria_label: label,
            onpointerenter: move |_| paused.set(true),
            onpointerleave: move |_| if focused.peek().is_none() { paused.set(false); },
            onfocusin: move |_| paused.set(true),
            onfocusout: move |_| { focused.set(None); paused.set(false); },
            ..attributes,
            // Both regions exist, empty, before any message: a live region created with
            // its content is announced unreliably.
            ol { class: TOASTER.element("list"), role: "status", aria_live: "polite", {render(false)} }
            ol { class: TOASTER.element("list"), role: "alert", aria_live: "assertive", {render(true)} }
        }
    }
}

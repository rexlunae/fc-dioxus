//! Host access stays here so components keep renderer-independent signatures.
//!
//! Eval-capable hosts use a fixed script and a data channel. SSR and native hosts
//! report `Unsupported`, allowing callers to keep their native markup fallback.

use super::id::Id;
use dioxus::prelude::*;
use std::fmt;

/// Capabilities detected after mount, never used to change the initial tree.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Capabilities {
    /// The host implements modal dialogs.
    pub modal: bool,
    /// The host implements the popover API.
    pub popover: bool,
    /// The host implements clipboard writes.
    pub clipboard: bool,
}

/// A user preference the host reports through a media query. Fixed queries only, so no
/// caller can pass the bridge arbitrary text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum MediaPreference {
    /// `prefers-color-scheme: dark`.
    DarkScheme,
    /// `prefers-contrast: more`.
    MoreContrast,
    /// `forced-colors: active`.
    ForcedColors,
}
impl MediaPreference {
    const fn query(self) -> &'static str {
        match self {
            Self::DarkScheme => "(prefers-color-scheme: dark)",
            Self::MoreContrast => "(prefers-contrast: more)",
            Self::ForcedColors => "(forced-colors: active)",
        }
    }
}

/// A host operation could not be performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DomError {
    /// SSR, native, or a host without the requested capability.
    Unsupported,
    /// The target ID does not exist.
    MissingTarget,
    /// The host rejected the request (for example clipboard permissions).
    Rejected,
}
impl fmt::Display for DomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "host operation: {self:?}")
    }
}
impl std::error::Error for DomError {}

/// The position to which a node is scrolled.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScrollBlock {
    /// Nearest visible edge, avoiding unnecessary motion.
    #[default]
    Nearest,
    /// Block-start edge.
    Start,
    /// Center of the visible area.
    Center,
}

/// Which side of a popup sits at a point's x.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Edge {
    /// The popup's left edge.
    Left,
    /// The popup's right edge.
    Right,
}

/// A viewport point a popup opens at, in CSS pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenuPoint {
    /// Horizontal position.
    pub x: f64,
    /// Vertical position.
    pub y: f64,
    /// Where the popup's bottom goes if it would cross the viewport's, e.g. a
    /// trigger's top.
    pub flip_y: Option<f64>,
    /// Which edge sits at `x`; the inline start when `None`.
    pub edge: Option<Edge>,
}

/// An asynchronous host bridge; errors are acknowledged by the host.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DomBridge;

impl DomBridge {
    async fn send(operation: &str, data: &str, option: &str) -> Result<(), DomError> {
        let eval = document::eval(include_str!("platform.js"));
        eval.send((operation, data, option))
            .map_err(|_| DomError::Unsupported)?;
        match eval
            .join::<String>()
            .await
            .map_err(|_| DomError::Unsupported)?
            .as_str()
        {
            "ok" => Ok(()),
            "missing" => Err(DomError::MissingTarget),
            "unsupported" => Err(DomError::Unsupported),
            _ => Err(DomError::Rejected),
        }
    }
    /// Open a dialog modally after mount. Fall back to an inline open dialog.
    /// # Errors
    /// Returns a host error when the target or API is unavailable.
    pub async fn show_modal(self, id: &Id) -> Result<(), DomError> {
        Self::send("modal", id.as_str(), "").await
    }
    /// Close a dialog. Fall back to removing its open state in the virtual DOM.
    /// # Errors
    /// Returns a host error when the target or API is unavailable.
    pub async fn close_dialog(self, id: &Id) -> Result<(), DomError> {
        Self::send("close", id.as_str(), "").await
    }
    /// Change popover visibility. Fall back to an in-flow panel.
    /// # Errors
    /// Returns a host error when the target or API is unavailable.
    pub async fn toggle_popover(self, id: &Id, open: bool) -> Result<(), DomError> {
        Self::send("popover", id.as_str(), if open { "open" } else { "close" }).await
    }
    /// Focus a node. Prefer `MountedData::set_focus` when a mount handle exists.
    /// # Errors
    /// Returns a host error when the target or API is unavailable.
    pub async fn focus(self, id: &Id) -> Result<(), DomError> {
        Self::send("focus", id.as_str(), "").await
    }
    /// Scroll a node into view; unsupported hosts leave the viewport unchanged.
    /// # Errors
    /// Returns a host error when the target or API is unavailable.
    pub async fn scroll_into_view(self, id: &Id, block: ScrollBlock) -> Result<(), DomError> {
        Self::send(
            "scroll",
            id.as_str(),
            match block {
                ScrollBlock::Nearest => "nearest",
                ScrollBlock::Start => "start",
                ScrollBlock::Center => "center",
            },
        )
        .await
    }
    /// Whether a scroll container is at its inline start and end, read from the host the
    /// way upstream's `carousel.ts` does, RTL included.
    /// # Errors
    /// Returns a host error when the target or API is unavailable.
    pub async fn scroll_edges(self, id: &Id) -> Result<(bool, bool), DomError> {
        let eval = document::eval(include_str!("platform.js"));
        eval.send(("edges", id.as_str(), ""))
            .map_err(|_| DomError::Unsupported)?;
        let answer = eval
            .join::<String>()
            .await
            .map_err(|_| DomError::Unsupported)?;
        match answer.split_once(' ') {
            Some((start, end)) => Ok((start == "true", end == "true")),
            None if answer == "missing" => Err(DomError::MissingTarget),
            None => Err(DomError::Unsupported),
        }
    }
    /// Scroll a container by most of its visible width, forwards in the reading direction
    /// or back.
    /// # Errors
    /// Returns a host error when the target or API is unavailable.
    pub async fn page(self, id: &Id, forward: bool) -> Result<(), DomError> {
        Self::send("page", id.as_str(), if forward { "next" } else { "prev" }).await
    }
    /// Place a popup list under its invoker, or above it when there is no room below,
    /// capped to the space available. Positioning needs layout, so it is the host's.
    /// # Errors
    /// Returns a host error when the target or API is unavailable.
    pub async fn place_below(self, list: &Id, invoker: &Id) -> Result<(), DomError> {
        Self::send("place", list.as_str(), invoker.as_str()).await
    }
    /// Place a fixed popup at a viewport point, as upstream's `menu.ts` does: its start
    /// edge at `x` (or `edge`), clamped inside the viewport, and moved above `flip_y`
    /// when it would cross the bottom.
    /// # Errors
    /// Returns a host error when the target or API is unavailable.
    pub async fn place_at(self, id: &Id, point: MenuPoint) -> Result<(), DomError> {
        let edge = match point.edge {
            Some(Edge::Left) => "left",
            Some(Edge::Right) => "right",
            None => "",
        };
        let flip = point.flip_y.map(|f| f.to_string()).unwrap_or_default();
        Self::send(
            "point",
            id.as_str(),
            &format!("{},{},{flip},{edge}", point.x, point.y),
        )
        .await
    }
    /// The point a menu opens at under a trigger: its start edge, 4px below, flipping to
    /// 4px above.
    /// # Errors
    /// Returns a host error when the target or API is unavailable.
    pub async fn point_below(self, trigger: &Id) -> Result<MenuPoint, DomError> {
        let eval = document::eval(include_str!("platform.js"));
        eval.send(("below", trigger.as_str(), ""))
            .map_err(|_| DomError::Unsupported)?;
        let answer = eval
            .join::<String>()
            .await
            .map_err(|_| DomError::Unsupported)?;
        let parts: Vec<&str> = answer.split(',').collect();
        match parts.as_slice() {
            [x, y, flip, edge] => Ok(MenuPoint {
                x: x.parse().map_err(|_| DomError::Rejected)?,
                y: y.parse().map_err(|_| DomError::Rejected)?,
                flip_y: flip.parse().ok(),
                edge: Some(if *edge == "right" {
                    Edge::Right
                } else {
                    Edge::Left
                }),
            }),
            _ if answer == "missing" => Err(DomError::MissingTarget),
            _ => Err(DomError::Unsupported),
        }
    }
    /// Route a pointer's events to an element for the rest of a drag.
    /// # Errors
    /// Returns a host error when the target or API is unavailable.
    pub async fn capture_pointer(self, id: &Id, pointer: i32) -> Result<(), DomError> {
        Self::send("capture", id.as_str(), &pointer.to_string()).await
    }
    /// Show a checkbox as mixed, which only a DOM property can express. Without a host
    /// the checkbox renders unchecked.
    /// # Errors
    /// Returns a host error when the target or API is unavailable.
    pub async fn set_indeterminate(self, id: &Id, mixed: bool) -> Result<(), DomError> {
        Self::send(
            "indeterminate",
            id.as_str(),
            if mixed { "on" } else { "off" },
        )
        .await
    }
    /// Copy text through a data channel. On failure, retain selectable source text.
    /// # Errors
    /// Returns `Unsupported` or `Rejected` when clipboard access is unavailable.
    pub async fn copy_text(self, text: &str) -> Result<(), DomError> {
        Self::send("copy", text, "").await
    }
    /// Wait `ms` milliseconds on the host's clock, for timed behaviour such as toast
    /// expiry. Hosts without a script runtime return `Unsupported` at once, and callers
    /// keep their untimed fallback (a toast then stays until dismissed).
    /// # Errors
    /// Returns `Unsupported` when the host cannot run timers.
    pub async fn delay(self, ms: u32) -> Result<(), DomError> {
        let eval = document::eval(
            "const ms = await dioxus.recv(); await new Promise(r => setTimeout(r, ms)); return 'ok';",
        );
        eval.send(ms).map_err(|_| DomError::Unsupported)?;
        match eval.join::<String>().await {
            Ok(_) => Ok(()),
            Err(_) => Err(DomError::Unsupported),
        }
    }
    /// Whether a user preference is on, or `None` on hosts without media queries, where
    /// callers keep their light, standard-contrast default.
    pub async fn prefers(self, preference: MediaPreference) -> Option<bool> {
        let eval = document::eval(
            "const query = await dioxus.recv(); return typeof matchMedia === 'function' ? matchMedia(query).matches : null;",
        );
        eval.send(preference.query()).ok()?;
        eval.join::<Option<bool>>().await.ok().flatten()
    }
    /// Detect available host APIs after mounting; SSR and native return defaults.
    pub async fn capabilities(self) -> Capabilities {
        let eval = document::eval("return [typeof HTMLDialogElement !== 'undefined' && 'showModal' in HTMLDialogElement.prototype, 'showPopover' in HTMLElement.prototype, !!navigator.clipboard?.writeText];");
        let (modal, popover, clipboard) =
            eval.join::<(bool, bool, bool)>().await.unwrap_or_default();
        Capabilities {
            modal,
            popover,
            clipboard,
        }
    }
}

/// Read the web entrypoint's query string. Non-web hosts supply initial props.
/// This is input to the application, not capability-dependent component markup.
#[must_use]
pub fn location_query() -> Option<String> {
    #[cfg(all(feature = "web", target_arch = "wasm32"))]
    {
        web_sys::window().and_then(|window| window.location().search().ok())
    }
    #[cfg(not(all(feature = "web", target_arch = "wasm32")))]
    {
        None
    }
}

/// Something upstream's editor binding reported.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EditorEvent {
    /// The document changed; its HTML.
    Change(String),
    /// The marks covering the whole selection.
    Marks(Vec<crate::models::rich_text::Mark>),
}

/// Upstream's editor binding attached to one `.fc-editor`. Events arrive in order;
/// dropping the session stops listening but leaves the binding on its element.
pub struct EditorSession(document::Eval);
impl EditorSession {
    /// The next event, or `None` once the host stops sending.
    pub async fn next(&mut self) -> Option<EditorEvent> {
        let (kind, html, marks) = self
            .0
            .recv::<(String, String, Vec<crate::models::rich_text::Mark>)>()
            .await
            .ok()?;
        match kind.as_str() {
            "change" => Some(EditorEvent::Change(html)),
            _ => Some(EditorEvent::Marks(marks)),
        }
    }
    /// Replace the document. The HTML must come from the application's own state,
    /// never from untrusted input: like upstream's `value` setter, this is `innerHTML`.
    pub fn set_value(&self, html: &str) {
        let _ = self.0.send(html);
    }
}

impl DomBridge {
    /// Attach upstream's editor binding (`editor.ts`, shipped unchanged as JavaScript)
    /// to the `.fc-editor` with this id.
    /// # Errors
    /// `Unsupported` where the host cannot load modules (server rendering, native), and
    /// `MissingTarget` when no element has the id. The editor then stays plain,
    /// editable text with an inert toolbar, which upstream calls degraded but not broken.
    pub async fn bind_editor(self, id: &Id) -> Result<EditorSession, DomError> {
        let mut eval = document::eval(
            r#"const [id, base] = await dioxus.recv();
const root = document.getElementById(id);
if (!root) { dioxus.send(["missing", "", []]); return; }
const { FutureCssEditor } = await import(`${base}/editor.js`);
const editor = new FutureCssEditor({
  root,
  onChange: html => dioxus.send(["change", html, []]),
  onSelectionChange: active => dioxus.send(["marks", "", Object.keys(active).filter(mark => active[mark])]),
});
dioxus.send(["ready", "", []]);
while (true) editor.value = await dioxus.recv();"#,
        );
        eval.send((id.as_str(), super::assets::BINDINGS_JS.to_string()))
            .map_err(|_| DomError::Unsupported)?;
        match eval
            .recv::<(String, String, Vec<crate::models::rich_text::Mark>)>()
            .await
        {
            Ok((kind, _, _)) if kind == "ready" => Ok(EditorSession(eval)),
            Ok(_) => Err(DomError::MissingTarget),
            Err(_) => Err(DomError::Unsupported),
        }
    }
}

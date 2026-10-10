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

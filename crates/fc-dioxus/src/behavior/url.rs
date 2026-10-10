//! Which addresses an embedded frame may load: a port of `isSafeFrameUrl` in upstream's
//! `webview.ts`.
//!
//! An allowlist, because a webview's address bar is a navigation primitive: `javascript:`
//! or `data:` in it is a script-injection sink aimed at the host page. Unknown schemes
//! fail closed.

/// Schemes a frame may load.
const SAFE_SCHEMES: [&str; 3] = ["http", "https", "about"];

/// Why an address was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// Nothing to load.
    Empty,
    /// A scheme outside the allowlist, such as `javascript` or `data`.
    Scheme,
}

/// The address to load for what the user typed, or why it was refused.
///
/// Matches upstream's allowlist, after the cleaning the URL standard does first: leading
/// and trailing controls and spaces are trimmed and tabs and newlines removed anywhere,
/// so `java\nscript:` is caught as `javascript:`. Two address-bar conveniences go
/// beyond upstream, which resolves everything against the host page: a bare host such
/// as `example.com` or `localhost:8080` gets `https://`, and `//host` takes `https:`.
/// Paths, queries, and fragments still resolve against the page.
///
/// ```
/// use fc_dioxus::behavior::url::{frame_url, Refusal};
/// assert_eq!(frame_url("https://example.com/docs"), Ok("https://example.com/docs".into()));
/// assert_eq!(frame_url("example.com"), Ok("https://example.com".into()));
/// assert_eq!(frame_url("  JavaScript:alert(1)"), Err(Refusal::Scheme));
/// assert_eq!(frame_url("java\nscript:alert(1)"), Err(Refusal::Scheme));
/// ```
///
/// # Errors
/// [`Refusal::Empty`] for blank input; [`Refusal::Scheme`] for a scheme outside
/// `http`, `https`, and `about`.
pub fn frame_url(raw: &str) -> Result<String, Refusal> {
    let cleaned: String = raw
        .trim_matches(|c: char| c <= ' ')
        .chars()
        .filter(|c| !matches!(c, '\t' | '\n' | '\r'))
        .collect();
    if cleaned.is_empty() {
        return Err(Refusal::Empty);
    }
    if let Some(rest) = cleaned.strip_prefix("//") {
        return Ok(format!("https://{rest}"));
    }
    if cleaned.starts_with(['/', '.', '?', '#']) {
        return Ok(cleaned);
    }
    match scheme(&cleaned) {
        Some(scheme) if SAFE_SCHEMES.contains(&scheme.to_ascii_lowercase().as_str()) => Ok(cleaned),
        Some(_) => Err(Refusal::Scheme),
        None => Ok(format!("https://{cleaned}")),
    }
}

/// Whether the address is served over TLS, for the address bar's lock. `None` for
/// addresses with no transport, such as `about:blank`.
#[must_use]
pub fn is_secure(url: &str) -> Option<bool> {
    match scheme(url).map(str::to_ascii_lowercase).as_deref() {
        Some("https") => Some(true),
        Some("http") => Some(false),
        _ => None,
    }
}

/// The scheme, when the address has one: a letter, then letters, digits, `+`, `-`, or
/// `.`, then a colon. `host:8080` is a host and port, not a scheme.
fn scheme(url: &str) -> Option<&str> {
    let (candidate, rest) = url.split_once(':')?;
    let mut chars = candidate.chars();
    let valid = chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'));
    let port = rest
        .split(['/', '?', '#'])
        .next()
        .is_some_and(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()));
    (valid && !port).then_some(candidate)
}

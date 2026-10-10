//! Frame address safety, against `isSafeFrameUrl` in upstream's webview.ts plus the
//! address-bar conveniences documented on `frame_url`.
#![forbid(unsafe_code)]

use fc_dioxus::behavior::url::{frame_url, is_secure, Refusal};

#[test]
fn the_allowlist_admits_web_and_about_addresses() {
    for ok in [
        "https://example.com",
        "http://example.com/a?b#c",
        "about:blank",
        "HTTPS://EXAMPLE.COM",
    ] {
        assert_eq!(frame_url(ok).as_deref(), Ok(ok), "{ok}");
    }
}

#[test]
fn every_other_scheme_fails_closed() {
    for bad in [
        "javascript:alert(1)",
        "data:text/html,x",
        "vbscript:x",
        "file:///etc/passwd",
        "blob:x",
        "mailto:a@b",
        "chrome://settings",
    ] {
        assert_eq!(frame_url(bad), Err(Refusal::Scheme), "{bad}");
    }
}

#[test]
fn cleaning_happens_before_the_check_so_hidden_schemes_are_caught() {
    assert_eq!(frame_url("\u{0}  javascript:x  "), Err(Refusal::Scheme));
    assert_eq!(frame_url("java\tscr\nipt:x"), Err(Refusal::Scheme));
    assert_eq!(frame_url("JaVaScRiPt:x"), Err(Refusal::Scheme));
    assert_eq!(frame_url(" \n "), Err(Refusal::Empty));
}

#[test]
fn bare_hosts_get_https_and_relative_addresses_resolve_against_the_page() {
    assert_eq!(
        frame_url("example.com/docs").as_deref(),
        Ok("https://example.com/docs")
    );
    assert_eq!(
        frame_url("localhost:8080").as_deref(),
        Ok("https://localhost:8080")
    );
    assert_eq!(
        frame_url("//cdn.example.com/x").as_deref(),
        Ok("https://cdn.example.com/x")
    );
    assert_eq!(frame_url("/docs").as_deref(), Ok("/docs"));
    assert_eq!(frame_url("#top").as_deref(), Ok("#top"));
}

#[test]
fn only_tls_addresses_get_the_lock() {
    assert_eq!(is_secure("https://a"), Some(true));
    assert_eq!(is_secure("http://a"), Some(false));
    assert_eq!(is_secure("about:blank"), None);
}

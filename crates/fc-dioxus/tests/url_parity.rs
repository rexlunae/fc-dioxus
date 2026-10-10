//! `frame_url` against upstream's own `isSafeFrameUrl`, run by Node from the pinned
//! snapshot in `upstream/`. Only absolute addresses are compared: for relative ones and bare hosts the
//! port deliberately differs (documented on `frame_url`), since upstream resolves them
//! against the host page.
#![forbid(unsafe_code)]

use fc_dioxus::behavior::url::frame_url;
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

/// A JSON string literal; every non-alphanumeric character is escaped.
fn json(text: &str) -> String {
    let body: String = text
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_string()
            } else {
                format!("\\u{:04x}", u32::from(c))
            }
        })
        .collect();
    format!("\"{body}\"")
}

const INPUTS: &[&str] = &[
    "https://example.com/docs",
    "http://example.com/a?b#c",
    "about:blank",
    "HTTPS://EXAMPLE.COM",
    "javascript:alert(1)",
    "JaVaScRiPt:alert(1)",
    "java\nscript:alert(1)",
    "  javascript:alert(1)",
    "\u{0}javascript:x",
    "data:text/html,x",
    "vbscript:x",
    "file:///etc/passwd",
    "blob:https://example.com/x",
    "mailto:a@example.com",
    "chrome://settings",
    "ftp://example.com/x",
];

#[test]
fn absolute_addresses_get_the_same_verdict_as_upstream() -> Result<(), Box<dyn std::error::Error>> {
    // Upstream's file is part of the committed snapshot (`upstream/`, kept in step with
    // the pin by `cargo xtask sync-upstream`), so a fresh clone can run this.
    let module = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../upstream/webview.ts");
    let script = format!(
        "import {{ isSafeFrameUrl }} from {path:?}; let text = ''; for await (const chunk of process.stdin) text += chunk; console.log(JSON.stringify(JSON.parse(text).map(u => isSafeFrameUrl(u))));",
        path = module.to_string_lossy(),
    );
    let inputs = format!(
        "[{}]",
        INPUTS.iter().map(|u| json(u)).collect::<Vec<_>>().join(",")
    );
    let mut child = Command::new("node")
        .args(["--no-warnings", "--input-type=module", "-e", &script])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("no stdin")?
        .write_all(inputs.as_bytes())?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let upstream: Vec<bool> = String::from_utf8(output.stdout)?
        .trim()
        .trim_matches(['[', ']'])
        .split(',')
        .map(|v| v == "true")
        .collect();
    let ours: Vec<bool> = INPUTS.iter().map(|u| frame_url(u).is_ok()).collect();
    for ((input, theirs), mine) in INPUTS.iter().zip(&upstream).zip(&ours) {
        assert_eq!(mine, theirs, "verdicts differ for {input:?}");
    }
    assert_eq!(upstream.len(), INPUTS.len());
    Ok(())
}

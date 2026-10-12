//! The shipped stylesheets decode identically under any charset.
//!
//! A host that serves CSS without a charset into a page that is not UTF-8 decodes
//! the stylesheet as Latin-1. Upstream's glyphs survive that only while they stay
//! CSS escapes, so both the vendored bytes and the bundler's handling of them matter.
#![forbid(unsafe_code)]

use std::{error::Error, fs, path::Path};

type Outcome = Result<(), Box<dyn Error>>;

fn stylesheets(dir: &Path, out: &mut Vec<std::path::PathBuf>) -> Outcome {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            stylesheets(&path, out)?;
        } else if path.extension().is_some_and(|ext| ext == "css") {
            out.push(path);
        }
    }
    Ok(())
}

fn without_comments(css: &str) -> String {
    let mut out = String::new();
    let mut rest = css;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        rest = rest[start + 2..]
            .split_once("*/")
            .map_or("", |(_, after)| after);
    }
    out.push_str(rest);
    out
}

#[test]
fn vendored_rules_are_ascii() -> Outcome {
    let mut paths = Vec::new();
    stylesheets(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("assets"),
        &mut paths,
    )?;
    assert!(!paths.is_empty());
    for path in paths {
        let css = fs::read_to_string(&path)?;
        let rules = without_comments(&css);
        let glyph = rules.chars().find(|c| !c.is_ascii());
        assert_eq!(
            glyph,
            None,
            "{} has a raw glyph outside comments; upstream should escape it",
            path.display()
        );
    }
    Ok(())
}

#[test]
fn stylesheet_assets_are_not_minified() -> Outcome {
    // dx's minifier decodes "\2713" into raw UTF-8, which is exactly the byte
    // sequence that renders as "âœ“" under Latin-1.
    let assets =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/core/assets.rs"))?;
    let stylesheets = assets.matches(".css\"").count();
    assert!(stylesheets > 0);
    assert_eq!(
        assets
            .matches("AssetOptions::css().with_minify(false)")
            .count(),
        stylesheets,
        "every stylesheet asset keeps upstream's escapes"
    );
    Ok(())
}

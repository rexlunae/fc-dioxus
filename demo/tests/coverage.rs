//! Enforce exact CSS inventory, unknown-class rejection, and implemented states.
#![forbid(unsafe_code)]

use fc_a11y::Document;
use fc_dioxus::prelude::*;
use std::{collections::BTreeSet, fs, path::Path};

fn css_classes(css: &str) -> BTreeSet<String> {
    let mut code = String::new();
    let mut rest = css;
    while let Some((before, comment)) = rest.split_once("/*") {
        code.push_str(before);
        rest = comment.split_once("*/").map_or("", |(_, tail)| tail);
    }
    code.push_str(rest);
    code.split(".fc-")
        .skip(1)
        .map(|part| {
            format!(
                "fc-{}",
                part.chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
                    .collect::<String>()
            )
        })
        .collect()
}

#[test]
fn inventory_and_rendered_classes_cannot_silently_drift() -> Result<(), Box<dyn std::error::Error>>
{
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let all = css_classes(&fs::read_to_string(
        root.join("crates/fc-dioxus/assets/future-css.css"),
    )?);
    let html = dioxus_ssr::render_element(
        rsx! { fc_demo::Demo { options: fc_demo::DemoOptions { fixtures: true, ..fc_demo::DemoOptions::default() }, stylesheets: Stylesheets::External } },
    );
    let doc = Document::parse(&html);
    let emitted: BTreeSet<_> = doc
        .nodes
        .iter()
        .filter_map(|node| node.attr("class"))
        .flat_map(str::split_whitespace)
        .filter(|class| class.starts_with("fc-"))
        .map(str::to_owned)
        .collect();
    assert!(
        emitted.is_subset(&all),
        "unknown classes: {:?}",
        emitted.difference(&all).collect::<Vec<_>>()
    );
    let implemented_files = [
        "divider.css",
        "button.css",
        "card.css",
        "badge.css",
        "alert.css",
        "avatar.css",
        "spinner.css",
        "loading.css",
        "breadcrumb.css",
        "empty-state.css",
        "description-list.css",
        "list.css",
        "check.css",
        "field.css",
        "form-variants.css",
        "disclosure.css",
        "button-group.css",
        "control-group.css",
        "segmented.css",
        "split-button.css",
        "fab.css",
        "masthead.css",
        "navbar.css",
        "pagination.css",
        "search.css",
    ];
    for filename in implemented_files {
        let implemented = css_classes(&fs::read_to_string(
            root.join("crates/fc-dioxus/assets/components")
                .join(filename),
        )?);
        assert!(
            implemented.is_subset(&emitted),
            "unexercised {filename} classes: {:?}",
            implemented.difference(&emitted).collect::<Vec<_>>()
        );
    }
    let states: BTreeSet<_> = doc
        .nodes
        .iter()
        .filter(|node| node.has_class("fc-divider"))
        .map(|node| node.attr("class").unwrap_or("").to_owned())
        .collect();
    assert_eq!(
        states.len(),
        6,
        "each orientation/inset combination must be in fixtures"
    );
    let pending = fs::read_to_string(root.join("coverage-pending.toml"))?;
    let pending: BTreeSet<_> = pending
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix('"')
                .and_then(|line| line.strip_suffix("\","))
        })
        .map(str::to_owned)
        .collect();
    let mut expected = pending;
    for filename in implemented_files {
        assert!(
            expected.insert(filename.to_owned()),
            "{filename} must be removed from pending"
        );
    }
    let actual: BTreeSet<_> = fs::read_dir(root.join("crates/fc-dioxus/assets/components"))?
        .map(|entry| entry.map(|e| e.file_name().to_string_lossy().into_owned()))
        .collect::<Result<_, _>>()?;
    assert_eq!(
        expected, actual,
        "classify every new upstream stylesheet explicitly"
    );
    Ok(())
}

#[test]
fn generated_snapshot_is_unchanged() -> Result<(), Box<dyn std::error::Error>> {
    let status = std::process::Command::new("node")
        .arg("tools/xtask.mjs")
        .arg("verify-snapshot")
        .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join(".."))
        .status()?;
    assert!(status.success(), "run cargo xtask sync-upstream");
    Ok(())
}

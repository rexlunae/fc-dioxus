//! The diff view draws upstream's fc-diff output with every non-colour cue.
#![forbid(unsafe_code)]

use fc_dioxus::prelude::*;

#[test]
fn every_change_carries_a_glyph_a_label_and_data_fc_diff() {
    let html = dioxus_ssr::render_element(rsx! {
        FutureCss { stylesheets: Stylesheets::External,
            DiffView { before: "a\nb\n", after: "a\nc\n" }
        }
    });
    assert!(html.contains(r#"data-fc-diff="add""#), "{html}");
    assert!(html.contains(r#"data-fc-diff="remove""#), "{html}");
    assert!(html.contains(r#"aria-label="Added: c""#), "{html}");
    assert!(html.contains(r#"aria-label="Removed: b""#), "{html}");
    assert!(html.contains("+1") && html.contains("−1"), "{html}");
}

#[test]
fn folded_lines_stay_in_the_dom() {
    let before: String = (1..=20).map(|n| n.to_string() + "\n").collect();
    let after = before.replace("10\n", "ten\n");
    let html = dioxus_ssr::render_element(rsx! {
        FutureCss { stylesheets: Stylesheets::External,
            DiffView { before: before.clone(), after, context: 1 }
        }
    });
    assert!(html.contains("unchanged lines"), "{html}");
    assert_eq!(
        html.matches("fc-diff__line").count(),
        21,
        "every line is still rendered"
    );
}

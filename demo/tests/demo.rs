//! Every shipped component has a live source example and a fixture entry.
#![forbid(unsafe_code)]

use fc_a11y::{Audit, Document};
use fc_demo::{Demo, DemoOptions, COMPONENTS, DIVIDER_EXAMPLES};
use fc_dioxus::prelude::*;

fn page(fixtures: bool) -> String {
    dioxus_ssr::render_element(
        rsx! { Demo { options: DemoOptions { fixtures, ..DemoOptions::default() }, stylesheets: Stylesheets::External } },
    )
}

#[test]
fn every_public_component_is_registered_and_shown() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../crates/fc-dioxus/src");
    let mut paths = vec![root.join("theme.rs")];
    for entry in std::fs::read_dir(root.join("components"))? {
        let path = entry?.path();
        if path.extension().is_some_and(|ext| ext == "rs") {
            paths.push(path);
        }
    }
    for path in paths {
        let source = std::fs::read_to_string(path)?;
        for segment in source.split("#[component]").skip(1) {
            if let Some(name) = segment
                .trim_start()
                .strip_prefix("pub fn ")
                .and_then(|s| s.split('(').next())
            {
                assert!(
                    COMPONENTS.iter().any(|(component, _)| *component == name),
                    "public component {name} has no demo registration"
                );
            }
        }
    }
    let showcase = Document::parse(&page(false));
    let fixture = Document::parse(&page(true));
    for (_, section) in COMPONENTS {
        assert!(
            showcase
                .nodes
                .iter()
                .any(|node| node.attr("data-showcase") == Some(section)),
            "missing showcase {section}"
        );
        assert!(
            fixture
                .nodes
                .iter()
                .any(|node| node.attr("data-fixture") == Some(section)),
            "missing fixture {section}"
        );
    }
    for example in DIVIDER_EXAMPLES {
        assert!(example.source.contains("// example:start"));
        assert!(example.source.contains("#[component]"));
        let rendered = dioxus_ssr::render_element((example.render)());
        assert!(rendered.contains("fc-divider"));
    }
    Ok(())
}

#[test]
fn both_pages_pass_all_implemented_static_rules() {
    for fixtures in [false, true] {
        let report = Audit.run(&Document::parse(&page(fixtures)));
        assert_eq!(report.errors(), 0, "{:#?}", report.findings);
    }
}

#[test]
fn query_inputs_select_only_known_theme_and_direction_values() {
    let options = DemoOptions::from_query("?fixtures=true&theme=dark&dir=rtl");
    assert_eq!(
        options,
        DemoOptions {
            fixtures: true,
            theme: Theme::Dark,
            direction: Direction::Rtl
        }
    );
    assert_eq!(
        DemoOptions::from_query("?theme=garbage&dir=sideways&token=secret"),
        DemoOptions::default()
    );
}

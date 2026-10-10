//! The design and palette catalogs against upstream's at the pin, and the root attributes.
#![forbid(unsafe_code)]

use fc_dioxus::prelude::*;

const PRESETS: &str = include_str!("../../../upstream/theme-presets.ts");
const THEMES_CSS: &str = include_str!("../assets/themes.css");

/// The value of every `key:'value'` pair in one exported array, in source order.
fn field(array: &str, key: &str) -> Vec<String> {
    let start = format!("export const {array} = [");
    let body = PRESETS
        .split_once(&start)
        .and_then(|(_, rest)| rest.split_once("] as const;"))
        .map_or("", |(body, _)| body);
    let needle = format!("{key}:'");
    body.split(&needle)
        .skip(1)
        .filter_map(|rest| rest.split_once('\'').map(|(value, _)| value.to_owned()))
        .collect()
}

#[test]
fn designs_match_upstream_catalog_order_names_and_paired_palettes() {
    let ids = field("themePresets", "id");
    assert_eq!(ids.len(), 21, "upstream changed its design catalog");
    let ours: Vec<_> = Design::ALL.iter().map(|d| d.as_str().to_owned()).collect();
    assert_eq!(ours, ids);
    let names: Vec<_> = Design::ALL.iter().map(|d| d.name().to_owned()).collect();
    assert_eq!(names, field("themePresets", "name"));
    let descriptions: Vec<_> = Design::ALL
        .iter()
        .map(|d| d.description().to_owned())
        .collect();
    assert_eq!(descriptions, field("themePresets", "description"));
    let palettes: Vec<_> = Design::ALL
        .iter()
        .map(|d| d.palette().as_str().to_owned())
        .collect();
    assert_eq!(palettes, field("themePresets", "palette"));
}

#[test]
fn palettes_match_upstream_catalog() {
    let ours: Vec<_> = Palette::ALL.iter().map(|p| p.as_str().to_owned()).collect();
    assert_eq!(ours, field("themePalettes", "id"));
    let names: Vec<_> = Palette::ALL.iter().map(|p| p.name().to_owned()).collect();
    assert_eq!(names, field("themePalettes", "name"));
}

#[test]
fn every_design_and_palette_has_a_rule_in_the_shipped_css() {
    for design in Design::ALL {
        let selector = format!("[data-fc-design=\"{}\"]", design.as_str());
        assert!(THEMES_CSS.contains(&selector), "missing {selector}");
        assert_eq!(Design::from_attribute(design.as_str()), Some(*design));
    }
    for palette in Palette::ALL {
        let selector = format!("[data-fc-palette=\"{}\"]", palette.as_str());
        assert!(THEMES_CSS.contains(&selector), "missing {selector}");
        assert_eq!(Palette::from_attribute(palette.as_str()), Some(*palette));
    }
    assert_eq!(Design::from_attribute("custom"), None);
    assert_eq!(Palette::from_attribute(""), None);
}

fn root(design: Option<Design>, palette: Option<Palette>) -> String {
    dioxus_ssr::render_element(rsx! {
        FutureCss { design, palette, stylesheets: Stylesheets::External, p { "x" } }
    })
}

#[test]
fn a_design_applies_its_paired_palette_unless_one_is_chosen() {
    let html = root(Some(Design::Ocean), None);
    assert!(html.contains("data-fc-design=\"ocean\""), "{html}");
    assert!(html.contains("data-fc-palette=\"teal\""), "{html}");

    let html = root(Some(Design::Ocean), Some(Palette::Rose));
    assert!(html.contains("data-fc-palette=\"rose\""), "{html}");
    assert!(!html.contains("teal"), "{html}");

    let html = root(None, Some(Palette::Amber));
    assert!(!html.contains("data-fc-design"), "{html}");
    assert!(html.contains("data-fc-palette=\"amber\""), "{html}");

    // Neither set: a nested root inherits whatever surrounds it.
    let html = root(None, None);
    assert!(!html.contains("data-fc-design"), "{html}");
    assert!(!html.contains("data-fc-palette"), "{html}");
}

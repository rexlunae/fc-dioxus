//! Icons are decorative unless labelled, and icon-only buttons are always named.
#![forbid(unsafe_code)]

use fc_dioxus::prelude::*;

fn render(element: Element) -> String {
    dioxus_ssr::render_element(rsx! { FutureCss { stylesheets: Stylesheets::External, {element} } })
}

#[test]
fn every_icon_round_trips_through_its_upstream_name() {
    assert_eq!(
        IconName::ALL.len(),
        66,
        "upstream changed the set; update the docs"
    );
    for icon in IconName::ALL {
        assert_eq!(IconName::from_name(icon.as_str()), Some(*icon));
        assert!(icon.path().starts_with('M'), "{icon:?} has path data");
    }
    assert_eq!(IconName::from_name("not-an-icon"), None);
}

#[test]
fn icons_are_hidden_unless_labelled_and_mirror_only_along_the_text() {
    let decorative = render(rsx! { Icon { name: IconName::Copy } });
    assert!(decorative.contains(r#"aria-hidden="true""#), "{decorative}");
    assert!(!decorative.contains("role="), "{decorative}");
    assert!(!decorative.contains("fc-icon--mirror"), "{decorative}");

    let labelled = render(rsx! { Icon { name: IconName::Error, label: name!("Build failed") } });
    assert!(labelled.contains(r#"role="img""#), "{labelled}");
    assert!(
        labelled.contains(r#"aria-label="Build failed""#),
        "{labelled}"
    );
    assert!(!labelled.contains("aria-hidden"), "{labelled}");

    let back = render(rsx! { Icon { name: IconName::ArrowStart, size: IconSize::Lg } });
    assert!(
        back.contains("fc-icon--mirror") && back.contains("fc-icon--lg"),
        "{back}"
    );
}

#[test]
fn icon_buttons_carry_the_name_on_the_button() {
    let html = render(rsx! {
        IconButton { icon: IconName::Copy, label: name!("Copy"), variant: ButtonVariant::Ghost, "data-copy": "snippet" }
    });
    assert!(
        html.contains("fc-btn--icon") && html.contains("fc-btn--ghost"),
        "{html}"
    );
    assert!(html.contains(r#"aria-label="Copy""#), "{html}");
    assert!(
        html.contains(r#"data-copy="snippet""#),
        "consumer attributes reach the button: {html}"
    );
    assert!(html.contains("fc-icon"), "{html}");

    let busy = render(
        rsx! { IconButton { icon: IconName::Refresh, label: name!("Refresh"), busy: true } },
    );
    assert!(
        busy.contains(r#"aria-busy="true""#) && busy.contains("fc-btn__spinner"),
        "{busy}"
    );
    assert!(
        !busy.contains("fc-icon"),
        "the spinner replaces the icon: {busy}"
    );
}

//! Regression and false-positive coverage for the initial rule catalogue.
#![forbid(unsafe_code)]
#![cfg(feature = "html")]

use fc_a11y::{Audit, Document, Severity};

#[test]
fn every_rule_has_a_catch_and_a_legitimate_pattern() {
    for (rule, bad, good) in [
        (
            "img-alt",
            "<img src=x>",
            "<img src=x alt=''><img src=y alt='A bird'><img src=z aria-label='A tree'><img src=t title='A bridge'>",
        ),
        (
            "idref",
            "<input aria-labelledby=missing>",
            "<label id=name for=input>Name</label><input id=input aria-labelledby=name>",
        ),
        (
            "duplicate-id",
            "<span id=n>A</span><span id=n>B</span><input aria-labelledby=n>",
            "<span id=n>A</span><span id=n>B</span>",
        ),
        (
            "tabindex-positive",
            "<button tabindex=2>Save</button>",
            "<button tabindex=0>Save</button><p tabindex=-1>Heading</p>",
        ),
    ] {
        assert!(
            Audit
                .run(&Document::parse(bad))
                .findings
                .iter()
                .any(|finding| finding.rule == rule),
            "{rule} missed its violation"
        );
        assert!(
            !Audit
                .run(&Document::parse(good))
                .findings
                .iter()
                .any(|finding| finding.rule == rule),
            "{rule} rejected a legitimate pattern"
        );
    }
}

#[test]
fn positive_tabindex_requests_review_without_claiming_a_definite_wcag_failure() {
    let report = Audit.run(&Document::parse("<button tabindex=1>Next</button>"));
    assert_eq!(report.errors(), 0);
    assert!(report
        .findings
        .iter()
        .all(|finding| finding.severity == Severity::Warning));
}

#[test]
fn html5_parser_decodes_entities_and_repairs_optional_end_tags() {
    let doc =
        Document::parse("<ul><li id='a&amp;b'>One<li>Two</ul><p aria-labelledby='a&amp;b'>Three");
    assert_eq!(doc.nodes.iter().filter(|node| node.tag == "li").count(), 2);
    assert_eq!(Audit.run(&doc).errors(), 0);
}

#[test]
fn images_hidden_by_ancestors_are_not_exposed_alternative_failures() {
    let report = Audit.run(&Document::parse("<div aria-hidden='true'><img src=x></div><div hidden><img src=y></div><div inert><img src=z></div>"));
    assert_eq!(report.errors(), 0);
}

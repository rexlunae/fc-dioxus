//! Generated tables must have an entry for every enum variant.
#![forbid(unsafe_code)]

use fc_tokens::{Scheme, Token};

#[test]
fn every_token_has_a_unique_name_and_both_scheme_values() {
    let mut names = std::collections::BTreeSet::new();
    for token in Token::ALL {
        assert!(token.name().starts_with("--fc-"));
        assert!(names.insert(token.name()), "duplicate token name");
        assert_eq!(token.css_var(), format!("var({})", token.name()));
        assert!(!token.value(Scheme::Light).is_empty());
        assert!(!token.value(Scheme::Dark).is_empty());
    }
    assert!(!names.is_empty());
}

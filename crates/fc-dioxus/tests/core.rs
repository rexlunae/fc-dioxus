//! Consumer-input validation, semantic protection, and deterministic hydration IDs.
#![forbid(unsafe_code)]

use fc_a11y::Document;
use fc_dioxus::prelude::*;
use std::{cell::RefCell, rc::Rc};

#[test]
fn validated_text_and_identifiers_reject_bad_input() {
    assert!(AccessibleName::try_from(" \n\t").is_err());
    assert_eq!(
        AccessibleName::try_from(" Close ").map(|n| n.to_string()),
        Ok("Close".to_owned())
    );
    for bad in ["", "a b", "a\nb", "a\0b"] {
        assert!(Id::try_from(bad).is_err());
    }
    assert!(Id::try_from("meaningful-id").is_ok());
}

#[test]
fn modifier_declarations_round_trip_and_preserve_consumer_classes() {
    for variant in DividerInset::ALL {
        assert_eq!(variant.to_string().parse::<DividerInset>(), Ok(*variant));
    }
    assert!("not-a-modifier".parse::<DividerInset>().is_err());
    assert_eq!(
        ClassList::new(DividerInset::BEM.block())
            .with_modifier(DividerInset::BEM, DividerInset::Middle)
            .extend(Some("consumer-class"))
            .to_string(),
        "fc-divider fc-divider--middle consumer-class"
    );
}

#[test]
fn consumer_attributes_cannot_remove_separator_semantics() {
    let html = dioxus_ssr::render_element(rsx! {
        FutureCss { stylesheets: Stylesheets::External,
            Divider { orientation: Orientation::Vertical, role: "button", aria_orientation: "horizontal", tabindex: "0", class: "extra", id: "separator" }
        }
    });
    let document = Document::parse(&html);
    let node = document
        .nodes
        .iter()
        .find(|node| node.attr("id") == Some("separator"));
    assert!(node.is_some());
    if let Some(node) = node {
        assert_eq!(node.attr("role"), Some("separator"));
        assert_eq!(node.attr("aria-orientation"), Some("vertical"));
        assert!(node.attr("tabindex").is_none());
        assert!(node.has_class("fc-divider") && node.has_class("extra"));
    }
}

#[component]
fn IdPair() -> Element {
    let one = use_id();
    let two = use_id();
    rsx! { label { id: "{one}", r#for: "{two}", "Label" } input { id: "{two}", aria_labelledby: "{one}" } }
}
fn ids_app() -> Element {
    rsx! {
        FutureCss { stylesheets: Stylesheets::External, IdPair {} IdPair {} }
        FutureCss { stylesheets: Stylesheets::External, IdPair {} }
    }
}
#[test]
fn ids_are_unique_between_roots_and_deterministic_between_ssr_renders() {
    let render = || {
        let mut dom = VirtualDom::new(ids_app);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    };
    let first = render();
    assert_eq!(first, render());
    let document = Document::parse(&first);
    let ids: Vec<_> = document
        .nodes
        .iter()
        .filter_map(|node| node.attr("id"))
        .collect();
    assert_eq!(ids.len(), 6);
    assert_eq!(
        ids.iter().collect::<std::collections::BTreeSet<_>>().len(),
        6
    );
    assert_eq!(fc_a11y::Audit.run(&document).errors(), 0);
}

#[test]
fn ids_also_remain_unique_without_a_provider() {
    let html = dioxus_ssr::render_element(rsx! { IdPair {} });
    let document = Document::parse(&html);
    let ids: Vec<_> = document
        .nodes
        .iter()
        .filter_map(|node| node.attr("id"))
        .collect();
    assert_eq!(ids.len(), 2);
    assert_eq!(
        ids.iter().collect::<std::collections::BTreeSet<_>>().len(),
        2
    );
}

#[derive(Clone)]
struct Capture(Rc<RefCell<Option<Controllable<u32>>>>);
#[derive(Clone)]
struct Changes(Rc<RefCell<Vec<u32>>>);

fn state_app() -> Element {
    let capture = use_context::<Capture>();
    let changes = use_context::<Changes>();
    let controlled = use_context::<bool>();
    let owner = use_signal(|| 3_u32);
    let state = use_controllable(
        controlled.then_some(owner.into()),
        || 2,
        Some(EventHandler::new(move |next| {
            changes.0.borrow_mut().push(next);
        })),
    );
    *capture.0.borrow_mut() = Some(state);
    rsx! { output { "{state.get()}" } }
}

#[test]
fn controlled_state_only_requests_changes_and_uncontrolled_state_stores_them() {
    for controlled in [false, true] {
        let capture = Capture(Rc::new(RefCell::new(None)));
        let changes = Changes(Rc::new(RefCell::new(Vec::new())));
        let mut dom = VirtualDom::new(state_app);
        dom.provide_root_context(capture.clone());
        dom.provide_root_context(changes.clone());
        dom.provide_root_context(controlled);
        dom.rebuild_in_place();
        let state = *capture.0.borrow();
        assert!(state.is_some());
        if let Some(state) = state {
            dom.in_scope(dioxus::core::ScopeId::APP, || state.set(8));
            assert_eq!(
                dom.in_runtime(|| state.get()),
                if controlled { 3 } else { 8 }
            );
            assert_eq!(*changes.0.borrow(), vec![8]);
        }
    }
}

#[test]
fn typed_contract_omits_absent_fields_and_keeps_values_valid() {
    let contract = FcData {
        kind: Some(Kind::Message),
        role: Some(MessageRole::Assistant),
        state: Some(Lifecycle::Streaming),
        seq: Some(7),
        ts: Some(Timestamp::from_epoch_millis(123)),
        ..FcData::default()
    };
    let attrs = contract.attributes();
    let html = dioxus_ssr::render_element(rsx! { div { ..attrs } });
    assert!(html.contains("data-fc-state=\"streaming\""));
    assert!(html.contains("data-fc-ts=\"123\""));
    assert!(!html.contains("data-fc-id"));
}

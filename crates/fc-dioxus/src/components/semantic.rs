//! Lifecycle-aware cards for agent interfaces: agent status, metric, transfer, approval,
//! user prompt, and device authorization, from their stylesheets of the same names.
//!
//! Each carries upstream's dual-audience contract: `data-fc-kind` names what it is, and
//! `data-fc-id`, `data-fc-seq`, `data-fc-state`, and `data-fc-ts` mirror the runtime record,
//! so automation can read the state without parsing text. Styling keys off
//! `data-fc-state`, the single source of truth for both audiences.
use crate::{
    core::{
        assets,
        attrs::{component_attributes, FcData, Kind},
    },
    prelude::*,
    theme::use_stylesheets,
};
const STATUS: Bem = Bem::new("fc-agent-status");
const METRIC: Bem = Bem::new("fc-metric");
const TRANSFER: Bem = Bem::new("fc-transfer");
const APPROVAL: Bem = Bem::new("fc-approval");
const PROMPT: Bem = Bem::new("fc-user-prompt");
const DEVICE: Bem = Bem::new("fc-device-auth");

/// The runtime record a card mirrors into `data-fc-*`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Record {
    /// Stable runtime id; never a credential.
    pub id: Option<Id>,
    /// Monotonic ordering.
    pub seq: Option<u64>,
    /// Lifecycle state, which also drives the styling.
    pub state: Option<Lifecycle>,
    /// When the record was produced or last changed.
    pub ts: Option<Timestamp>,
}
impl Record {
    fn attributes(&self, kind: Kind) -> Spread {
        FcData {
            kind: Some(kind),
            id: self.id.clone(),
            seq: self.seq,
            state: self.state,
            ts: self.ts.clone(),
            ..FcData::default()
        }
        .attributes()
    }
}

fn shell(
    bem: Bem,
    kind: Kind,
    label: &AccessibleName,
    record: &Record,
    class: Option<&str>,
    attributes: Vec<Attribute>,
) -> Spread {
    let mut attributes = component_attributes(
        attributes,
        ClassList::new(bem.block()).extend(class),
        &["aria-label"],
    );
    attributes.push(Attribute::new("aria-label", label.to_string(), None, false));
    attributes.extend(record.attributes(kind));
    attributes
}

/// One live line saying what an agent is doing, with a sparkle and an elapsed time.
#[component]
pub fn AgentStatus(
    state: Lifecycle,
    /// What is happening, announced politely as it changes.
    label: Element,
    #[props(default)] meta: Option<Element>,
    #[props(default = SpinnerKind::Sparkle)] spinner: SpinnerKind,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let mut attributes = component_attributes(
        attributes,
        ClassList::new(STATUS.block()).extend(class.as_deref()),
        &[],
    );
    attributes.extend(
        FcData {
            kind: Some(Kind::AgentStatus),
            state: Some(state),
            ..FcData::default()
        }
        .attributes(),
    );
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_AGENT_STATUS_CSS } }
        div { ..attributes,
            Spinner { kind: spinner, size: SpinnerSize::Sm, label: SpinnerLabel::Decorative }
            span { class: STATUS.element("label"), role: "status", {label} }
            if let Some(meta) = meta { span { class: STATUS.element("meta"), {meta} } }
        }
    }
}

/// A number worth watching, as a term and its value.
#[component]
pub fn Metric(
    label: AccessibleName,
    title: Element,
    /// The machine-readable number, written to `<data value>`. `None` when there is no
    /// value to report, and `display` says why.
    value: Option<f64>,
    /// How people read the value.
    display: String,
    #[props(default)] meta: Option<Element>,
    #[props(default)] actions: Option<Element>,
    /// A line above the term, e.g. the state in words.
    #[props(default)]
    status: Option<Element>,
    /// A standalone named region rather than one item among many.
    #[props(default)]
    region: bool,
    #[props(default)] record: Record,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let attributes = shell(
        METRIC,
        Kind::Metric,
        &label,
        &record,
        class.as_deref(),
        attributes,
    );
    let body = rsx! {
        {status}
        dl {
            dt { class: METRIC.element("title"), {title} }
            dd { class: METRIC.element("value"),
                if let Some(value) = value { data { value: "{value}", "{display}" } } else { "{display}" }
            }
            if let Some(meta) = meta { dd { class: METRIC.element("meta"), {meta} } }
        }
        if let Some(actions) = actions { div { class: METRIC.element("actions"), {actions} } }
    };
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_METRIC_CSS } }
        {card(region, attributes, body)}
    }
}

/// A file moving in or out, with progress while it runs.
#[component]
pub fn Transfer(
    label: AccessibleName,
    title: Element,
    #[props(default = HeadingLevel::H3)] heading: HeadingLevel,
    /// The file and how far along it is. Wrap names in `bdi` so mixed-direction text holds.
    #[props(default)]
    meta: Option<Element>,
    /// Progress while it runs, named by `label`.
    #[props(default)]
    progress: Option<ProgressValue>,
    #[props(default)] actions: Option<Element>,
    #[props(default)] status: Option<Element>,
    #[props(default)] region: bool,
    #[props(default)] record: Record,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let attributes = shell(
        TRANSFER,
        Kind::Transfer,
        &label,
        &record,
        class.as_deref(),
        attributes,
    );
    let body = rsx! {
        {status}
        {super::heading(heading, TRANSFER.element("title"), title)}
        if let Some(meta) = meta { p { class: TRANSFER.element("meta"), {meta} } }
        {children}
        if let Some(progress) = progress { Progress { label: label.clone(), value: progress } }
        if let Some(actions) = actions { div { class: TRANSFER.element("actions"), {actions} } }
    };
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_TRANSFER_CSS } }
        {card(region, attributes, body)}
    }
}

/// A request for the user's permission, e.g. to run a tool, with what exactly is asked.
#[component]
pub fn Approval(
    label: AccessibleName,
    title: Element,
    #[props(default = HeadingLevel::H3)] heading: HeadingLevel,
    #[props(default)] meta: Option<Element>,
    /// Allow and deny, as buttons. Deny comes first so it is not the default by position.
    #[props(default)]
    actions: Option<Element>,
    #[props(default)] status: Option<Element>,
    #[props(default)] region: bool,
    #[props(default)] record: Record,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let attributes = shell(
        APPROVAL,
        Kind::Approval,
        &label,
        &record,
        class.as_deref(),
        attributes,
    );
    let body = rsx! {
        {status}
        {super::heading(heading, APPROVAL.element("title"), title)}
        {children}
        if let Some(meta) = meta { p { class: APPROVAL.element("meta"), {meta} } }
        if let Some(actions) = actions { div { class: APPROVAL.element("actions"), {actions} } }
    };
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_APPROVAL_CSS } }
        {card(region, attributes, body)}
    }
}

/// A question an agent puts to the user, as a form. `onsubmit` receives nothing: read the
/// answer from the controls' own state. With `region` it is a plain named section, for
/// when there is nothing to submit.
#[component]
pub fn UserPrompt(
    label: AccessibleName,
    title: Element,
    #[props(default = HeadingLevel::H3)] heading: HeadingLevel,
    #[props(default)] meta: Option<Element>,
    #[props(default)] actions: Option<Element>,
    #[props(default)] onsubmit: Option<EventHandler<()>>,
    #[props(default)] status: Option<Element>,
    #[props(default)] region: bool,
    #[props(default)] record: Record,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let attributes = shell(
        PROMPT,
        Kind::UserPrompt,
        &label,
        &record,
        class.as_deref(),
        attributes,
    );
    let body = rsx! {
        {status}
        {super::heading(heading, PROMPT.element("title"), title)}
        {children}
        if let Some(meta) = meta { p { class: PROMPT.element("meta"), {meta} } }
        if let Some(actions) = actions { div { class: PROMPT.element("actions"), {actions} } }
    };
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_USER_PROMPT_CSS } }
        if region {
            section { ..attributes, {body} }
        } else {
            form { onsubmit: move |event| { event.prevent_default(); if let Some(onsubmit) = onsubmit { onsubmit.call(()); } },
                ..attributes, {body}
            }
        }
    }
}

/// A device-code sign-in: where to go and the code to enter there.
///
/// The code is rendered as text only. It is a credential, so it never goes into an
/// attribute, where it would be serialized into logs and caches.
#[component]
pub fn DeviceAuth(
    label: AccessibleName,
    title: Element,
    #[props(default = HeadingLevel::H3)] heading: HeadingLevel,
    /// The code to enter, once one has been issued.
    #[props(default)]
    code: Option<String>,
    #[props(default)] meta: Option<Element>,
    #[props(default)] actions: Option<Element>,
    #[props(default)] status: Option<Element>,
    #[props(default)] region: bool,
    #[props(default)] record: Record,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let attributes = shell(
        DEVICE,
        Kind::DeviceAuth,
        &label,
        &record,
        class.as_deref(),
        attributes,
    );
    let body = rsx! {
        {status}
        {super::heading(heading, DEVICE.element("title"), title)}
        {children}
        if let Some(code) = code { code { class: DEVICE.element("code"), "{code}" } }
        if let Some(meta) = meta { p { class: DEVICE.element("meta"), {meta} } }
        if let Some(actions) = actions { div { class: DEVICE.element("actions"), {actions} } }
    };
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_DEVICE_AUTH_CSS } }
        {card(region, attributes, body)}
    }
}

/// `<section>` for a card that stands alone as a named region (an empty state, a single
/// expanded view), `<article>` for one item in a stream of them.
fn card(region: bool, attributes: Spread, body: Element) -> Element {
    if region {
        rsx! { section { ..attributes, {body} } }
    } else {
        rsx! { article { ..attributes, {body} } }
    }
}

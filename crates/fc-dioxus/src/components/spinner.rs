//! Indeterminate activity indicators and shimmering status text, from spinner.css.
use crate::{
    core::{assets, attrs::component_attributes},
    prelude::*,
    theme::use_stylesheets,
};
const BEM: Bem = Bem::new("fc-spinner");
const SHIMMER: Bem = Bem::new("fc-shimmer-text");
const HIDDEN: Bem = Bem::new("fc-u-visually-hidden");
fc_modifier! {
    /// Which drawing, chosen by what the work is.
    pub enum SpinnerKind for BEM {
        /// Generic loading: a page, a request, a save.
        Ring => None,
        /// A softer tapering arc.
        Arc => "arc",
        /// Someone is composing a reply.
        Dots => "dots",
        /// Something is live and listening.
        Pulse => "pulse",
        /// A stream is flowing.
        Bars => "bars",
        /// Background work with several parts.
        Orbit => "orbit",
        /// A model is thinking.
        Sparkle => "sparkle",
    }
}
fc_modifier! {
    /// Spinner sizes.
    pub enum SpinnerSize for BEM {
        /// Default.
        Md => None,
        /// Extra small, for inline use.
        Xs => "xs",
        /// Small.
        Sm => "sm",
        /// Large.
        Lg => "lg",
        /// Extra large.
        Xl => "xl",
    }
}
fc_modifier! {
    /// Spinner color.
    pub enum SpinnerTone for BEM {
        /// The accent color.
        Accent => None,
        /// Muted, for background work.
        Muted => "muted",
        /// The surrounding text color.
        Current => "current",
    }
}

/// What a spinner says. There is no default: a spinner either decorates text that already
/// says what is happening, or it is the status and has to say so in words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpinnerLabel {
    /// Visible text beside the spinner describes the work.
    Decorative,
    /// The spinner is the status; the name is announced politely.
    Status(AccessibleName),
}

/// An animated activity indicator. The drawing is always hidden from assistive technology.
#[component]
pub fn Spinner(
    #[props(default)] kind: SpinnerKind,
    #[props(default)] size: SpinnerSize,
    #[props(default)] tone: SpinnerTone,
    label: SpinnerLabel,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let classes = ClassList::new(BEM.block())
        .with_modifier(BEM, kind)
        .with_modifier(BEM, size)
        .with_modifier(BEM, tone)
        .extend(class.as_deref());
    let attributes = component_attributes(attributes, classes, &["aria-hidden", "role"]);
    // Dots and bars animate their child spans; the other drawings are pseudo-elements.
    let parts = match kind {
        SpinnerKind::Dots => 3,
        SpinnerKind::Bars => 4,
        _ => 0,
    };
    let drawing = rsx! {
        span { aria_hidden: "true", ..attributes,
            for index in 0..parts { span { key: "{index}" } }
        }
    };
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_SPINNER_CSS } }
        match label {
            SpinnerLabel::Decorative => drawing,
            SpinnerLabel::Status(name) => rsx! {
                span { role: "status", {drawing} span { class: HIDDEN.block(), "{name}" } }
            },
        }
    }
}

/// Status text with a shimmer sweeping across it, e.g. "Running tests".
#[component]
pub fn ShimmerText(
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let attributes = component_attributes(
        attributes,
        ClassList::new(SHIMMER.block()).extend(class.as_deref()),
        &[],
    );
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_SPINNER_CSS } }
        span { ..attributes, {children} }
    }
}

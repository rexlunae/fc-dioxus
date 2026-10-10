//! Root theme, stylesheet policy, and deterministic context.

use crate::core::{
    assets,
    attrs::theme_attributes,
    id::IdAllocator,
    platform::{Capabilities, DomBridge},
};
use dioxus::prelude::*;

/// A stock visual preset from the pinned upstream theme stylesheet.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Design {
    /// Upstream future preset.
    #[default]
    Future,
    /// Upstream clarity preset.
    Clarity,
    /// Upstream helix preset.
    Helix,
    /// Upstream forest preset.
    Forest,
    /// Upstream editorial preset.
    Editorial,
    /// Upstream graphite preset.
    Graphite,
    /// Upstream ocean preset.
    Ocean,
    /// Upstream ember preset.
    Ember,
    /// Upstream parchment preset.
    Parchment,
    /// Upstream blueprint preset.
    Blueprint,
    /// Upstream candy preset.
    Candy,
    /// Upstream nocturne preset.
    Nocturne,
    /// Upstream aurora preset.
    Aurora,
    /// Upstream solar preset.
    Solar,
    /// Upstream bloom preset.
    Bloom,
    /// Upstream garnet preset.
    Garnet,
    /// Upstream origami preset.
    Origami,
    /// Upstream arcade preset.
    Arcade,
    /// Upstream bubble preset.
    Bubble,
    /// Upstream sundown preset.
    Sundown,
    /// Upstream pride preset.
    Pride,
}
impl Design {
    /// Every stock preset supported by this pin.
    pub const ALL: &'static [Self] = &[
        Self::Future,
        Self::Clarity,
        Self::Helix,
        Self::Forest,
        Self::Editorial,
        Self::Graphite,
        Self::Ocean,
        Self::Ember,
        Self::Parchment,
        Self::Blueprint,
        Self::Candy,
        Self::Nocturne,
        Self::Aurora,
        Self::Solar,
        Self::Bloom,
        Self::Garnet,
        Self::Origami,
        Self::Arcade,
        Self::Bubble,
        Self::Sundown,
        Self::Pride,
    ];
    /// The upstream attribute value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Future => "future",
            Self::Clarity => "clarity",
            Self::Helix => "helix",
            Self::Forest => "forest",
            Self::Editorial => "editorial",
            Self::Graphite => "graphite",
            Self::Ocean => "ocean",
            Self::Ember => "ember",
            Self::Parchment => "parchment",
            Self::Blueprint => "blueprint",
            Self::Candy => "candy",
            Self::Nocturne => "nocturne",
            Self::Aurora => "aurora",
            Self::Solar => "solar",
            Self::Bloom => "bloom",
            Self::Garnet => "garnet",
            Self::Origami => "origami",
            Self::Arcade => "arcade",
            Self::Bubble => "bubble",
            Self::Sundown => "sundown",
            Self::Pride => "pride",
        }
    }
}

/// Upstream's supported schemes. System inherits the user's preference.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Theme {
    /// Follow the surrounding theme and system preference.
    #[default]
    System,
    /// Light scheme.
    Light,
    /// Dark scheme.
    Dark,
    /// High-contrast light scheme.
    HighContrastLight,
    /// High-contrast dark scheme.
    HighContrastDark,
}
impl Theme {
    /// The upstream attribute, absent in system mode.
    #[must_use]
    pub const fn attribute(self) -> Option<&'static str> {
        match self {
            Self::System => None,
            Self::Light => Some("light"),
            Self::Dark => Some("dark"),
            Self::HighContrastLight => Some("high-contrast-light"),
            Self::HighContrastDark => Some("high-contrast-dark"),
        }
    }
}
/// Upstream's optional brand palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Brand {
    /// The forest palette.
    Forest,
}
/// Inline text direction.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Direction {
    /// Left to right.
    #[default]
    Ltr,
    /// Right to left.
    Rtl,
}
impl Direction {
    /// HTML's `dir` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ltr => "ltr",
            Self::Rtl => "rtl",
        }
    }
}
/// Whether components inject their generated upstream stylesheet assets.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stylesheets {
    /// Root foundation files plus the stylesheets of rendered components.
    #[default]
    Automatic,
    /// The application already loads the upstream bundle.
    External,
}

pub(crate) fn use_stylesheets() -> bool {
    try_consume_context::<ReadSignal<Stylesheets>>()
        .is_none_or(|policy| *policy.read() == Stylesheets::Automatic)
}

/// Props for the root theme and CSS provider.
#[derive(Props, Clone, PartialEq)]
pub struct FutureCssProps {
    /// Optional stock visual preset; absence inherits the surrounding design.
    #[props(default)]
    pub design: Option<Design>,
    /// Scheme, inherited when omitted.
    #[props(default)]
    pub theme: Theme,
    /// Optional upstream brand.
    #[props(default)]
    pub brand: Option<Brand>,
    /// Inline direction, inherited when omitted.
    #[props(default)]
    pub direction: Option<Direction>,
    /// Set to external if the app already loads the CSS bundle.
    #[props(default)]
    pub stylesheets: Stylesheets,
    /// The application's component tree.
    pub children: Element,
}

/// Provide the pinned stylesheets and theme without changing pre-mount structure.
#[component]
pub fn FutureCss(props: FutureCssProps) -> Element {
    use_context_provider(IdAllocator::new);
    let mut policy = use_signal(|| props.stylesheets);
    if *policy.peek() != props.stylesheets {
        policy.set(props.stylesheets);
    }
    use_context_provider(|| ReadSignal::from(policy));
    let capabilities = use_signal(Capabilities::default);
    use_context_provider(|| capabilities);
    let mut capabilities = capabilities;
    let attrs = theme_attributes(
        props.theme.attribute(),
        props.brand.map(|brand| match brand {
            Brand::Forest => "forest",
        }),
        props.design.map(Design::as_str),
    );
    rsx! {
        if props.stylesheets == Stylesheets::Automatic {
            document::Stylesheet { href: assets::LAYERS_CSS }
            document::Stylesheet { href: assets::TOKENS_CSS }
            document::Stylesheet { href: assets::RESET_CSS }
            document::Stylesheet { href: assets::BASE_CSS }
            document::Stylesheet { href: assets::LAYOUT_CSS }
            document::Stylesheet { href: assets::TYPE_CSS }
            document::Stylesheet { href: assets::UTILITIES_CSS }
            document::Stylesheet { href: assets::THEMES_CSS }
        }
        div {
            dir: props.direction.map(Direction::as_str),
            onmounted: move |_| async move { capabilities.set(DomBridge.capabilities().await); },
            ..attrs,
            {props.children}
        }
    }
}

//! Root theme, stylesheet policy, and deterministic context.

use crate::core::{
    assets,
    attrs::{theme_attributes, theme_pairs},
    id::IdAllocator,
    platform::{Capabilities, DomBridge},
};
use dioxus::prelude::*;
use std::{
    cell::Cell,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
};

// The two catalogs below mirror upstream's `src/js/theme-presets.ts` at the pin, in the
// same order, so the demo lists them as upstream's own picker does. `tests/themes.rs`
// compares them against the synced copy in `upstream/theme-presets.ts`.

/// A stock visual preset: geometry, elevation, surface treatment, and type.
///
/// A design carries no colors of its own. Colors come from a [`Palette`], and each design
/// names the palette upstream pairs it with ([`Design::palette`]), which `FutureCss` applies
/// unless a palette is chosen explicitly.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Design {
    /// Future: Balanced corners, quiet surfaces and soft elevation.
    #[default]
    Future,
    /// Clarity: Crisp edges, flat surfaces and outlined actions.
    Clarity,
    /// Helix: Charcoal chrome, mint actions and violet feature surfaces.
    Helix,
    /// Forest: Organic curves, green accents and calm surfaces.
    Forest,
    /// Editorial: Serif reading, restrained corners and ink-like controls.
    Editorial,
    /// Graphite: Monochrome surfaces with distinct, structured edges.
    Graphite,
    /// Ocean: Cool teal, rounded panels and a tinted header.
    Ocean,
    /// Ember: Warm accents, substantial elevation and squared panels.
    Ember,
    /// Blueprint: Square geometry, blue accents and precise outlines.
    Blueprint,
    /// Bloom: Playful curves, rose accents and floating colored panels.
    Bloom,
    /// Aurora: Violet ribbons, luminous curves and floating surfaces.
    Aurora,
    /// Candy: Rose candy stripes, pillowy panels and playful elevation.
    Candy,
    /// Solar: Amber sunshine, square panels and decisive offset shadows.
    Solar,
    /// Nocturne: Ink chrome, violet spotlights and elegant serif reading.
    Nocturne,
    /// Arcade: Green scanlines, square geometry and terminal-inspired type.
    Arcade,
    /// Parchment: Warm ruled paper, serif type and quiet outlined controls.
    Parchment,
    /// Bubble: Teal bubbles, generous curves and buoyant layered shadows.
    Bubble,
    /// Garnet: Rose jewel facets, crisp serif panels and charcoal chrome.
    Garnet,
    /// Origami: Blue folded ribbons, angular outlines and paper shadows.
    Origami,
    /// Sundown: Ember horizons, rounded reading panels and long soft shadows.
    Sundown,
    /// Pride: Rainbow ribbons, joyful rounded panels and clear violet actions.
    Pride,
}
impl Design {
    /// Every stock design at this pin, in upstream's catalog order.
    pub const ALL: &'static [Self] = &[
        Self::Future,
        Self::Clarity,
        Self::Helix,
        Self::Forest,
        Self::Editorial,
        Self::Graphite,
        Self::Ocean,
        Self::Ember,
        Self::Blueprint,
        Self::Bloom,
        Self::Aurora,
        Self::Candy,
        Self::Solar,
        Self::Nocturne,
        Self::Arcade,
        Self::Parchment,
        Self::Bubble,
        Self::Garnet,
        Self::Origami,
        Self::Sundown,
        Self::Pride,
    ];
    /// The `data-fc-design` value.
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
            Self::Blueprint => "blueprint",
            Self::Bloom => "bloom",
            Self::Aurora => "aurora",
            Self::Candy => "candy",
            Self::Solar => "solar",
            Self::Nocturne => "nocturne",
            Self::Arcade => "arcade",
            Self::Parchment => "parchment",
            Self::Bubble => "bubble",
            Self::Garnet => "garnet",
            Self::Origami => "origami",
            Self::Sundown => "sundown",
            Self::Pride => "pride",
        }
    }
    /// Upstream's display name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Future => "Future",
            Self::Clarity => "Clarity",
            Self::Helix => "Helix",
            Self::Forest => "Forest",
            Self::Editorial => "Editorial",
            Self::Graphite => "Graphite",
            Self::Ocean => "Ocean",
            Self::Ember => "Ember",
            Self::Blueprint => "Blueprint",
            Self::Bloom => "Bloom",
            Self::Aurora => "Aurora",
            Self::Candy => "Candy",
            Self::Solar => "Solar",
            Self::Nocturne => "Nocturne",
            Self::Arcade => "Arcade",
            Self::Parchment => "Parchment",
            Self::Bubble => "Bubble",
            Self::Garnet => "Garnet",
            Self::Origami => "Origami",
            Self::Sundown => "Sundown",
            Self::Pride => "Pride",
        }
    }
    /// Upstream's one-line description.
    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            Self::Future => "Balanced corners, quiet surfaces and soft elevation.",
            Self::Clarity => "Crisp edges, flat surfaces and outlined actions.",
            Self::Helix => "Charcoal chrome, mint actions and violet feature surfaces.",
            Self::Forest => "Organic curves, green accents and calm surfaces.",
            Self::Editorial => "Serif reading, restrained corners and ink-like controls.",
            Self::Graphite => "Monochrome surfaces with distinct, structured edges.",
            Self::Ocean => "Cool teal, rounded panels and a tinted header.",
            Self::Ember => "Warm accents, substantial elevation and squared panels.",
            Self::Blueprint => "Square geometry, blue accents and precise outlines.",
            Self::Bloom => "Playful curves, rose accents and floating colored panels.",
            Self::Aurora => "Violet ribbons, luminous curves and floating surfaces.",
            Self::Candy => "Rose candy stripes, pillowy panels and playful elevation.",
            Self::Solar => "Amber sunshine, square panels and decisive offset shadows.",
            Self::Nocturne => "Ink chrome, violet spotlights and elegant serif reading.",
            Self::Arcade => "Green scanlines, square geometry and terminal-inspired type.",
            Self::Parchment => "Warm ruled paper, serif type and quiet outlined controls.",
            Self::Bubble => "Teal bubbles, generous curves and buoyant layered shadows.",
            Self::Garnet => "Rose jewel facets, crisp serif panels and charcoal chrome.",
            Self::Origami => "Blue folded ribbons, angular outlines and paper shadows.",
            Self::Sundown => "Ember horizons, rounded reading panels and long soft shadows.",
            Self::Pride => "Rainbow ribbons, joyful rounded panels and clear violet actions.",
        }
    }
    /// The palette upstream pairs with this design.
    #[must_use]
    // One arm per design in catalog order reads against upstream line by line; merging
    // the arms that share a palette would hide which design a pairing belongs to.
    #[allow(clippy::match_same_arms)]
    pub const fn palette(self) -> Palette {
        match self {
            Self::Future => Palette::Default,
            Self::Clarity => Palette::Default,
            Self::Helix => Palette::Helix,
            Self::Forest => Palette::Forest,
            Self::Editorial => Palette::Ink,
            Self::Graphite => Palette::Ink,
            Self::Ocean => Palette::Teal,
            Self::Ember => Palette::Ember,
            Self::Blueprint => Palette::Blue,
            Self::Bloom => Palette::Rose,
            Self::Aurora => Palette::Violet,
            Self::Candy => Palette::Rose,
            Self::Solar => Palette::Amber,
            Self::Nocturne => Palette::Violet,
            Self::Arcade => Palette::Forest,
            Self::Parchment => Palette::Amber,
            Self::Bubble => Palette::Teal,
            Self::Garnet => Palette::Rose,
            Self::Origami => Palette::Blue,
            Self::Sundown => Palette::Ember,
            Self::Pride => Palette::Violet,
        }
    }
    /// Parse a `data-fc-design` value.
    #[must_use]
    pub fn from_attribute(value: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|design| design.as_str() == value)
    }
}

/// A stock color palette: every semantic color role, in light and dark.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Palette {
    /// Future blue.
    #[default]
    Default,
    /// Blueprint blue.
    Blue,
    /// Violet.
    Violet,
    /// Forest.
    Forest,
    /// Ink.
    Ink,
    /// Ocean teal.
    Teal,
    /// Ember.
    Ember,
    /// Rose.
    Rose,
    /// Amber.
    Amber,
    /// Helix violet & mint.
    Helix,
}
impl Palette {
    /// Every stock palette at this pin, in upstream's catalog order.
    pub const ALL: &'static [Self] = &[
        Self::Default,
        Self::Blue,
        Self::Violet,
        Self::Forest,
        Self::Ink,
        Self::Teal,
        Self::Ember,
        Self::Rose,
        Self::Amber,
        Self::Helix,
    ];
    /// The `data-fc-palette` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Blue => "blue",
            Self::Violet => "violet",
            Self::Forest => "forest",
            Self::Ink => "ink",
            Self::Teal => "teal",
            Self::Ember => "ember",
            Self::Rose => "rose",
            Self::Amber => "amber",
            Self::Helix => "helix",
        }
    }
    /// Upstream's display name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Default => "Future blue",
            Self::Blue => "Blueprint blue",
            Self::Violet => "Violet",
            Self::Forest => "Forest",
            Self::Ink => "Ink",
            Self::Teal => "Ocean teal",
            Self::Ember => "Ember",
            Self::Rose => "Rose",
            Self::Amber => "Amber",
            Self::Helix => "Helix violet & mint",
        }
    }
    /// Parse a `data-fc-palette` value.
    #[must_use]
    pub fn from_attribute(value: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|palette| palette.as_str() == value)
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
    /// Follow the user's system preference even inside a root that sets a scheme;
    /// [`Theme::System`] inherits the surrounding one instead.
    FollowSystem,
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
            Self::FollowSystem => Some("system"),
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
    /// Embed the generated upstream bundle for hosts without stylesheet asset loading.
    /// Includes every component stylesheet; the host's CSP must permit inline styles.
    Inline,
    /// The application already loads the upstream bundle.
    External,
}

/// The reading direction in effect, from the nearest [`FutureCss`] that set one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DirectionContext(Direction);

/// The reading direction components should use for arrow keys. Left-to-right unless a
/// [`FutureCss`] root above sets `direction`.
#[must_use]
pub fn use_direction() -> Direction {
    try_consume_context::<Signal<DirectionContext>>()
        .map_or(Direction::Ltr, |direction| direction.read().0)
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
    /// Optional color palette. When omitted, a set `design` applies its own paired
    /// palette, and with neither the surrounding palette is inherited.
    #[props(default)]
    pub palette: Option<Palette>,
    /// Scheme, inherited when omitted.
    #[props(default)]
    pub theme: Theme,
    /// Optional upstream brand.
    #[props(default)]
    pub brand: Option<Brand>,
    /// Inline direction, inherited when omitted.
    #[props(default)]
    pub direction: Option<Direction>,
    /// Set to external if the app already loads the CSS bundle, or inline for
    /// hosts without an asset resource provider.
    #[props(default)]
    pub stylesheets: Stylesheets,
    /// The application's component tree.
    pub children: Element,
}

// Names each outermost root to the document-side theme owner.
static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

/// Provide the pinned stylesheets and theme without changing pre-mount structure.
#[component]
pub fn FutureCss(props: FutureCssProps) -> Element {
    // Only the outermost root reaches <html>; checked before this root provides its own
    // direction context, which is what a nested root finds.
    let outermost = use_hook(|| try_consume_context::<Signal<DirectionContext>>().is_none());
    use_context_provider(IdAllocator::new);
    // A root without a direction inherits the one around it.
    let inherited = use_direction();
    let effective = DirectionContext(props.direction.unwrap_or(inherited));
    let mut direction = use_context_provider(|| Signal::new(effective));
    if *direction.peek() != effective {
        direction.set(effective);
    }
    let mut policy = use_signal(|| props.stylesheets);
    if *policy.peek() != props.stylesheets {
        policy.set(props.stylesheets);
    }
    use_context_provider(|| ReadSignal::from(policy));
    let capabilities = use_signal(Capabilities::default);
    use_context_provider(|| capabilities);
    let mut capabilities = capabilities;
    let pairs = theme_pairs(
        props.theme.attribute(),
        props.brand.map(|brand| match brand {
            Brand::Forest => "forest",
        }),
        props.design.map(Design::as_str),
        // A design without its palette restyles shapes but leaves every color at the
        // default, which reads as the theme failing to apply.
        props
            .palette
            .or(props.design.map(Design::palette))
            .map(Palette::as_str),
    );
    let attrs = theme_attributes(&pairs);
    // The scheme on this root's element cannot reach <html> and <body>, whose canvas
    // background then stays on the OS scheme around and beyond the content. Upstream
    // expects these attributes on <html>, as its own theme picker sets them.
    // Tokens are unique per process, which covers every document a process hosts.
    let token = use_hook(|| NEXT_ROOT.fetch_add(1, Ordering::Relaxed));
    let generation = use_hook(|| Rc::new(Cell::new(0_u64)));
    let next = generation.clone();
    use_effect(use_reactive!(|pairs| {
        if !outermost {
            return;
        }
        next.set(next.get() + 1);
        let generation = next.get();
        spawn(async move {
            let _ = DomBridge
                .mirror_document_theme(token, generation, Some(&pairs))
                .await;
        });
    }));
    use_drop(move || {
        if outermost {
            DomBridge.release_document_theme(token, generation.get() + 1);
        }
    });
    rsx! {
        if props.stylesheets == Stylesheets::Inline {
            // Text children also work on hosts without an HTML parser provider.
            // Only the pinned, generated bundle is embedded; never consumer CSS.
            style { {include_str!("../assets/future-css.css")} }
        }
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

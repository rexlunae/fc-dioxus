//! A theme picker from theme-picker.css: every design as a live preview card, and
//! selects for palette, scheme, and contrast.
//!
//! It chooses; [`FutureCss`] applies. The choice is a value the application feeds back
//! into its root, so the picker never writes to the document behind the tree's back.
//! Custom themes (upstream's editor, TOML import and export, and contrast validation)
//! wait on a Rust theme model upstream; see COMPONENTS.md, "Waiting on upstream".
use crate::{
    core::{
        assets,
        attrs::{component_attributes, theme_attributes, FcData, Kind, ThemePickerData},
        controlled::use_controllable,
        platform::{DomBridge, MediaPreference},
        state::Lifecycle,
    },
    prelude::*,
    theme::use_stylesheets,
};
const BEM: Bem = Bem::new("fc-theme-picker");
const CHECK: Bem = Bem::new("fc-check");
const INPUT: Bem = Bem::new("fc-input");

/// Light, dark, or the user's preference.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Scheme {
    /// Follow the system.
    #[default]
    System,
    /// Light.
    Light,
    /// Dark.
    Dark,
}
impl Scheme {
    /// Every scheme, in menu order.
    pub const ALL: [Self; 3] = [Self::System, Self::Light, Self::Dark];
    /// The value upstream records.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }
    const fn label(self) -> &'static str {
        match self {
            Self::System => "Follow system",
            Self::Light => "Light",
            Self::Dark => "Dark",
        }
    }
}

/// Standard contrast following the system, or high contrast.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Contrast {
    /// Follow the system.
    #[default]
    System,
    /// High contrast.
    High,
}
impl Contrast {
    /// Every setting, in menu order.
    pub const ALL: [Self; 2] = [Self::System, Self::High];
    /// The value upstream records.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::High => "high",
        }
    }
    const fn label(self) -> &'static str {
        match self {
            Self::System => "Follow system",
            Self::High => "High contrast",
        }
    }
}

/// Everything the picker chooses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ThemeChoice {
    /// The design preset.
    pub design: Design,
    /// The colour palette.
    pub palette: Palette,
    /// Light, dark, or system.
    pub scheme: Scheme,
    /// Standard or high contrast.
    pub contrast: Contrast,
}
impl Default for ThemeChoice {
    fn default() -> Self {
        Self::from_design(Design::default())
    }
}
impl ThemeChoice {
    /// A design with its own paired palette, following the system scheme and contrast.
    #[must_use]
    pub const fn from_design(design: Design) -> Self {
        Self {
            design,
            palette: design.palette(),
            scheme: Scheme::System,
            contrast: Contrast::System,
        }
    }
    /// The [`Theme`] for [`FutureCss`]. High contrast has only light and dark variants,
    /// so with the system scheme it needs to know which the system prefers.
    ///
    /// ```
    /// use fc_dioxus::prelude::*;
    /// let mut choice = ThemeChoice::default();
    /// assert_eq!(choice.theme(true), Theme::System);
    /// choice.contrast = Contrast::High;
    /// assert_eq!(choice.theme(true), Theme::HighContrastDark);
    /// choice.scheme = Scheme::Light;
    /// assert_eq!(choice.theme(true), Theme::HighContrastLight);
    /// ```
    #[must_use]
    pub const fn theme(self, prefers_dark: bool) -> Theme {
        let dark = match self.scheme {
            Scheme::System => prefers_dark,
            Scheme::Light => false,
            Scheme::Dark => true,
        };
        match (self.contrast, self.scheme, dark) {
            (Contrast::High, _, true) => Theme::HighContrastDark,
            (Contrast::High, _, false) => Theme::HighContrastLight,
            (Contrast::System, Scheme::System, _) => Theme::System,
            (Contrast::System, _, true) => Theme::Dark,
            (Contrast::System, _, false) => Theme::Light,
        }
    }
}

/// The theme picker. Picking a design also picks its paired palette, as upstream's
/// binding does; the palette select can then override it.
#[component]
pub fn ThemePicker(
    #[props(default)] value: Option<ReadSignal<ThemeChoice>>,
    #[props(default)] default_value: ThemeChoice,
    #[props(default)] onchange: Option<EventHandler<ThemeChoice>>,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let state = use_controllable(value, || default_value, onchange);
    let group = use_id();
    // Upstream resolves "follow system" with media queries; so do the previews, which
    // must show high contrast in the scheme the page will actually use.
    let mut dark = use_signal(|| false);
    let mut forced = use_signal(|| false);
    let mut more = use_signal(|| false);
    use_hook(move || {
        spawn(async move {
            dark.set(
                DomBridge
                    .prefers(MediaPreference::DarkScheme)
                    .await
                    .unwrap_or(false),
            );
            more.set(
                DomBridge
                    .prefers(MediaPreference::MoreContrast)
                    .await
                    .unwrap_or(false),
            );
            forced.set(
                DomBridge
                    .prefers(MediaPreference::ForcedColors)
                    .await
                    .unwrap_or(false),
            );
        });
    });
    let choice = state.get();
    let high = choice.contrast == Contrast::High || more() || forced();
    let shown = if dark() && choice.scheme == Scheme::System || choice.scheme == Scheme::Dark {
        "dark"
    } else {
        "light"
    };
    let status = format!(
        "{} · {shown}{}",
        choice.design.name(),
        if forced() {
            " · forced colors"
        } else if high {
            " · high contrast"
        } else {
            ""
        }
    );
    let preview_theme = match (high, shown) {
        (true, "dark") => "high-contrast-dark",
        (true, _) => "high-contrast-light",
        (false, scheme) => scheme,
    };

    let mut attributes = component_attributes(
        attributes,
        ClassList::new(BEM.block()).extend(class.as_deref()),
        &[],
    );
    attributes.extend(
        FcData {
            kind: Some(Kind::ThemePicker),
            state: Some(Lifecycle::Complete),
            ..FcData::default()
        }
        .attributes(),
    );
    attributes.extend([
        ThemePickerData::Design.attribute(choice.design.as_str()),
        ThemePickerData::Palette.attribute(choice.palette.as_str()),
        ThemePickerData::Scheme.attribute(choice.scheme.as_str()),
        ThemePickerData::Contrast.attribute(choice.contrast.as_str()),
    ]);
    let group = group.to_string();

    rsx! {
        if use_stylesheets() {
            document::Stylesheet { href: assets::COMPONENTS_CHECK_CSS }
            document::Stylesheet { href: assets::COMPONENTS_FIELD_CSS }
            document::Stylesheet { href: assets::COMPONENTS_THEME_PICKER_CSS }
        }
        div { ..attributes,
            fieldset { class: BEM.element("themes"),
                legend { class: BEM.element("legend"), "Design theme" }
                for design in Design::ALL.iter().copied() {
                    // Each card wears its own design and palette, so the preview is the
                    // theme itself rather than a picture of it.
                    label { key: "{design.as_str()}", class: BEM.element("option"),
                        ..theme_attributes(Some(preview_theme), None, Some(design.as_str()), Some(design.palette().as_str())),
                        input { r#type: "radio", class: CHECK.element("input"), name: "{group}", value: design.as_str(),
                            checked: choice.design == design,
                            onchange: move |_| state.set(ThemeChoice { design, palette: design.palette(), ..state.get() }),
                        }
                        span {
                            span { class: BEM.element("preview"), aria_hidden: "true",
                                span { class: BEM.element("window"),
                                    span { class: BEM.element("window-bar"), "Workspace" }
                                    span { class: BEM.element("window-body"),
                                        span { class: BEM.element("window-nav"), span { "Home" } span { "Projects" } }
                                        span { class: BEM.element("window-content"),
                                            span { class: BEM.element("window-feature"), "Your next idea" }
                                            span { class: BEM.element("preview-card"), "Ready to create" }
                                            span { class: BEM.element("preview-action"), "Continue" }
                                        }
                                    }
                                }
                            }
                            strong { class: BEM.element("name"), "{design.name()}" }
                            span { class: BEM.element("description"), "{design.description()}" }
                        }
                    }
                }
            }
            div { class: BEM.element("controls"),
                label { class: BEM.element("field"), "Color palette"
                    select { class: INPUT.block(), value: choice.palette.as_str(),
                        onchange: move |event| if let Some(palette) = Palette::from_attribute(&event.value()) {
                            state.set(ThemeChoice { palette, ..state.get() });
                        },
                        for palette in Palette::ALL.iter().copied() {
                            option { key: "{palette.as_str()}", value: palette.as_str(), selected: palette == choice.palette, "{palette.name()}" }
                        }
                    }
                }
                label { class: BEM.element("field"), "Color scheme"
                    select { class: INPUT.block(), value: choice.scheme.as_str(),
                        onchange: move |event| if let Some(scheme) = Scheme::ALL.into_iter().find(|s| s.as_str() == event.value()) {
                            state.set(ThemeChoice { scheme, ..state.get() });
                        },
                        for scheme in Scheme::ALL {
                            option { key: "{scheme.as_str()}", value: scheme.as_str(), selected: scheme == choice.scheme, "{scheme.label()}" }
                        }
                    }
                }
                label { class: BEM.element("field"), "Contrast"
                    select { class: INPUT.block(), value: choice.contrast.as_str(),
                        onchange: move |event| if let Some(contrast) = Contrast::ALL.into_iter().find(|c| c.as_str() == event.value()) {
                            state.set(ThemeChoice { contrast, ..state.get() });
                        },
                        for contrast in Contrast::ALL {
                            option { key: "{contrast.as_str()}", value: contrast.as_str(), selected: contrast == choice.contrast, "{contrast.label()}" }
                        }
                    }
                }
            }
            p { class: BEM.element("status"), role: "status", "{status}" }
        }
    }
}

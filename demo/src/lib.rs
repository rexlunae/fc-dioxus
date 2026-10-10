//! The same compiled examples feed the showcase, fixture page, and parity tests.
#![forbid(unsafe_code)]

use fc_dioxus::{
    core::{
        assets,
        platform::{self, DomBridge},
    },
    prelude::*,
};

pub mod examples;

const CONTAINER: Bem = Bem::new("fc-container");
const STACK: Bem = Bem::new("fc-stack");
const CLUSTER: Bem = Bem::new("fc-cluster");
const PAD: Bem = Bem::new("fc-u-p-6");
const SOURCE: Bem = Bem::new("fc-u-prewrap");
const GRID: Bem = Bem::new("fc-grid-auto");
const WIDE_GRID: Bem = Bem::new("fc-grid-auto--wide");
const SPACED: Bem = Bem::new("fc-stack--8");
const CHECK: Bem = Bem::new("fc-check__input");
const BETWEEN: Bem = Bem::new("fc-cluster--between");

/// A compiled example and the exact source that defines it.
#[derive(Clone, Copy)]
pub struct Example {
    /// Stable identifier used by the browser tests.
    pub id: &'static str,
    /// Human-readable example title.
    pub title: &'static str,
    /// Compiled render function.
    pub render: fn() -> Element,
    /// Source included from that function's file.
    pub source: &'static str,
}

/// Every implemented public component is represented here.
pub const COMPONENTS: &[(&str, &str)] = &[
    ("FutureCss", "future-css"),
    ("Divider", "divider"),
    ("Button", "button"),
    ("Badge", "badge"),
    ("Tag", "badge"),
    ("Kbd", "badge"),
    ("Alert", "alert"),
    ("Card", "card"),
    ("CardHeader", "card"),
    ("CardHeading", "card"),
    ("CardTitle", "card"),
    ("CardSubtitle", "card"),
    ("CardMedia", "card"),
    ("CardBody", "card"),
    ("CardLayout", "card"),
    ("CardFooter", "card"),
    ("CardLink", "card"),
];

/// The examples that render every Divider state.
pub const DIVIDER_EXAMPLES: &[Example] = &[
    Example {
        id: "horizontal",
        title: "Full, inset, and middle",
        render: || rsx! { examples::divider::Horizontal {} },
        source: include_str!("examples/divider/horizontal.rs"),
    },
    Example {
        id: "vertical",
        title: "A separator in a row",
        render: || rsx! { examples::divider::Vertical {} },
        source: include_str!("examples/divider/vertical.rs"),
    },
    Example {
        id: "vertical-insets",
        title: "Vertical inset variants",
        render: || rsx! { examples::divider::VerticalInsets {} },
        source: include_str!("examples/divider/vertical_insets.rs"),
    },
];

/// Styled component sections, rendered from the same files shown as Rust source.
pub const COMPONENT_EXAMPLES: &[Example] = &[
    Example {
        id: "button",
        title: "Buttons & actions",
        render: || rsx! { examples::button::States {} },
        source: include_str!("examples/button/states.rs"),
    },
    Example {
        id: "card",
        title: "Cards & surfaces",
        render: || rsx! { examples::card::States {} },
        source: include_str!("examples/card/states.rs"),
    },
    Example {
        id: "badge",
        title: "Badges, tags & keys",
        render: || rsx! { examples::badge::States {} },
        source: include_str!("examples/badge/states.rs"),
    },
    Example {
        id: "alert",
        title: "Alerts & feedback",
        render: || rsx! { examples::alert::States {} },
        source: include_str!("examples/alert/states.rs"),
    },
];

/// Initial application inputs, shared by SSR and client rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DemoOptions {
    /// Render every static state rather than the showcase.
    pub fixtures: bool,
    /// Initial scheme.
    pub theme: Theme,
    /// Initial stock visual preset.
    pub design: Design,
    /// Initial direction.
    pub direction: Direction,
}
impl Default for DemoOptions {
    fn default() -> Self {
        Self {
            fixtures: false,
            theme: Theme::System,
            design: Design::Helix,
            direction: Direction::Ltr,
        }
    }
}
impl DemoOptions {
    /// Read only known query keys and enum values, ignoring arbitrary input.
    #[must_use]
    pub fn from_query(query: &str) -> Self {
        let mut options = Self::default();
        for (key, value) in query
            .trim_start_matches('?')
            .split('&')
            .filter_map(|part| part.split_once('='))
        {
            match key {
                "fixtures" => options.fixtures = value == "true",
                "theme" => options.theme = parse_theme(value),
                "design" => options.design = parse_design(value),
                "dir" => {
                    options.direction = if value == "rtl" {
                        Direction::Rtl
                    } else {
                        Direction::Ltr
                    }
                }
                _ => {}
            }
        }
        options
    }
}
fn parse_design(value: &str) -> Design {
    Design::ALL
        .iter()
        .copied()
        .find(|design| design.as_str() == value)
        .unwrap_or(Design::Helix)
}
fn parse_theme(value: &str) -> Theme {
    match value {
        "light" => Theme::Light,
        "dark" => Theme::Dark,
        "high-contrast-light" => Theme::HighContrastLight,
        "high-contrast-dark" => Theme::HighContrastDark,
        _ => Theme::System,
    }
}

/// Webview/native entrypoint. SSR passes the same inputs to `Demo` explicitly.
#[component]
pub fn App() -> Element {
    let options = use_hook(|| {
        if cfg!(feature = "fullstack") {
            // A hydrated app starts from identical inputs on both sides. Request-aware
            // routing will supply these props when the fullstack demo is expanded.
            DemoOptions::default()
        } else {
            platform::location_query().map_or_else(DemoOptions::default, |query| {
                DemoOptions::from_query(&query)
            })
        }
    });
    rsx! { Demo { options } }
}

/// Props for the shared demo root.
#[derive(Props, Clone, PartialEq)]
pub struct DemoProps {
    /// Initial query inputs; the server and client must supply the same values.
    #[props(default)]
    pub options: DemoOptions,
    /// External stylesheet policy is useful for standalone SSR exports.
    #[props(default)]
    pub stylesheets: Stylesheets,
}

/// The renderer-independent showcase and fixture application.
#[component]
pub fn Demo(props: DemoProps) -> Element {
    let mut theme = use_signal(|| props.options.theme);
    let mut design = use_signal(|| props.options.design);
    let mut direction = use_signal(|| props.options.direction);
    let mut forest = use_signal(|| false);
    let fixtures = props.options.fixtures;
    rsx! {
        FutureCss { theme: theme(), design: design(), direction: direction(), brand: forest().then_some(Brand::Forest), stylesheets: props.stylesheets,
            if props.stylesheets == Stylesheets::Automatic { document::Stylesheet { href: assets::COMPONENTS_CHECK_CSS } }
            div { class: PAD.block(),
                main { class: ClassList::new(CONTAINER.block()).with(STACK.block()).with(SPACED.block()).to_string(),
                    header { class: ClassList::new(CLUSTER.block()).with(BETWEEN.block()).to_string(),
                        div { strong { "future-css" } " / Dioxus" }
                        nav { aria_label: "Demo pages", class: CLUSTER.block(),
                            a { href: "?fixtures=false", "Showcase" }
                            a { href: "?fixtures=true", "All fixture states" }
                            a { href: "https://github.com/rexlunae/future-css", "Upstream ↗" }
                        }
                    }
                    div { class: ClassList::new(GRID.block()).with(WIDE_GRID.block()).to_string(),
                        Card { variant: CardVariant::Feature,
                            p { "FUTURE CSS / DIOXUS" }
                            h1 { if fixtures { "Every state. Every detail." } else { "One library. Your kind of interface." } }
                            p { "Expressive surfaces, purposeful color, and accessible controls. Built with Rust. Styled entirely by future-css." }
                            div { class: CLUSTER.block(),
                                Button { variant: ButtonVariant::Secondary, onclick: move |_| {
                                    let index = Design::ALL.iter().position(|item| *item == design()).unwrap_or(0);
                                    if let Some(next) = Design::ALL.get((index + 1) % Design::ALL.len()) { design.set(*next); }
                                }, "Try another design" }
                                Badge { tone: BadgeTone::Solid, "21 stock themes" }
                            }
                            p { "Current design: " strong { "{design().as_str()}" } }
                        }
                        section { "data-showcase": "future-css", aria_labelledby: "appearance-title",
                            Card {
                                CardHeader { CardHeading {
                                    CardTitle { level: Some(HeadingLevel::H2), id: "appearance-title", "Design studio" }
                                    CardSubtitle { "Change the system. Every component follows." }
                                } }
                                CardBody {
                                    div { class: GRID.block(),
                                        div { label { r#for: "design", "Visual preset" }
                                            select { id: "design", value: design().as_str(), onchange: move |event| design.set(parse_design(&event.value())),
                                                for preset in Design::ALL { option { value: preset.as_str(), selected: *preset == design(), "{preset:?}" } }
                                            }
                                        }
                                        div { label { r#for: "theme", "Color scheme" }
                                            select { id: "theme", value: theme().attribute().unwrap_or("system"), onchange: move |event| theme.set(parse_theme(&event.value())),
                                                option { value: "system", selected: theme() == Theme::System, "System" }
                                                option { value: "light", selected: theme() == Theme::Light, "Light" }
                                                option { value: "dark", selected: theme() == Theme::Dark, "Dark" }
                                                option { value: "high-contrast-light", selected: theme() == Theme::HighContrastLight, "High contrast · light" }
                                                option { value: "high-contrast-dark", selected: theme() == Theme::HighContrastDark, "High contrast · dark" }
                                            }
                                        }
                                        div { label { r#for: "direction", "Reading direction" }
                                            select { id: "direction", value: direction().as_str(), onchange: move |event| direction.set(if event.value() == "rtl" { Direction::Rtl } else { Direction::Ltr }),
                                                option { value: "ltr", selected: direction() == Direction::Ltr, "Left to right" }
                                                option { value: "rtl", selected: direction() == Direction::Rtl, "Right to left" }
                                            }
                                        }
                                    }
                                    label { input { r#type: "checkbox", class: CHECK.block(), checked: forest(), onchange: move |event| forest.set(event.checked()) } " Forest brand override" }
                                }
                                CardFooter { divided: true, Badge { tone: BadgeTone::Success, dot: true, "Upstream styles" } Badge { tone: BadgeTone::Info, "Native semantics" } }
                            }
                            if !fixtures { Source { text: include_str!("examples/future_css/themes.rs"), name: "theme-source" } }
                        }
                    }
                    if fixtures {
                        section { "data-fixture": "future-css", aria_label: "Every root scheme and stock preset", examples::future_css::Themes {} }
                    }
                    for example in COMPONENT_EXAMPLES {
                        section { key: "{example.id}", id: example.id, "data-showcase": example.id, "data-fixture": fixtures.then_some(example.id),
                            aria_labelledby: "{example.id}-title", class: STACK.block(),
                            h2 { id: "{example.id}-title", "{example.title}" }
                            {(example.render)()}
                            if !fixtures { Source { text: example.source, name: example.id } }
                        }
                    }
                    section { "data-showcase": "divider", "data-fixture": fixtures.then_some("divider"), aria_labelledby: "divider-title", class: STACK.block(),
                        h2 { id: "divider-title", "Dividers" }
                        for example in DIVIDER_EXAMPLES {
                            article { key: "{example.id}", "data-example": example.id, class: STACK.block(),
                                h3 { "{example.title}" }
                                {(example.render)()}
                                if !fixtures { Source { text: example.source, name: example.id } }
                            }
                        }
                    }
                    footer { p { "future-css {assets::CSS_VERSION} · Dioxus 0.7 · Semantic HTML, shared design tokens, no local CSS." } }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct SourceProps {
    text: &'static str,
    name: &'static str,
}

#[component]
fn Source(props: SourceProps) -> Element {
    let mut message = use_signal(String::new);
    let source = props
        .text
        .split_once("// example:start\n")
        .map_or(props.text, |(_, shown)| shown)
        .trim();
    rsx! {
        details {
            summary { "Rust source" }
            pre { class: SOURCE.block(), code { "{source}" } }
            Button { variant: ButtonVariant::Secondary, size: ButtonSize::Sm, "data-copy": props.name,
                onclick: move |_| async move {
                    message.set(match DomBridge.copy_text(source).await {
                        Ok(()) => "Copied Rust source.".to_owned(),
                        Err(_) => "Copy is unavailable here. Select the source above to copy it.".to_owned(),
                    });
                },
                "Copy source"
            }
            p { role: "status", "{message}" }
        }
    }
}

/// Render a complete HTML document with external assets for static auditing.
#[cfg(feature = "ssr")]
#[must_use]
pub fn render_page(options: DemoOptions) -> String {
    let content =
        dioxus_ssr::render_element(rsx! { Demo { options, stylesheets: Stylesheets::External } });
    format!("<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>future-css · Dioxus</title><link rel=\"stylesheet\" href=\"/future-css.css\"></head><body class=\"fc-container fc-container--wide fc-u-p-0\">{content}</body></html>")
}

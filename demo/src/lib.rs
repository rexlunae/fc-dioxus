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
const BG: Bem = Bem::new("fc-u-bg-canvas");
const FG: Bem = Bem::new("fc-u-text-default");
const BUTTON: Bem = Bem::new("fc-btn");

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
pub const COMPONENTS: &[(&str, &str)] = &[("FutureCss", "future-css"), ("Divider", "divider")];

/// The examples that render every Divider state.
pub const DIVIDER_EXAMPLES: &[Example] = &[
    Example {
        id: "horizontal",
        title: "Full, inset, and middle",
        render: examples::divider::Horizontal,
        source: include_str!("examples/divider/horizontal.rs"),
    },
    Example {
        id: "vertical",
        title: "A separator in a row",
        render: examples::divider::Vertical,
        source: include_str!("examples/divider/vertical.rs"),
    },
    Example {
        id: "vertical-insets",
        title: "Vertical inset variants",
        render: examples::divider::VerticalInsets,
        source: include_str!("examples/divider/vertical_insets.rs"),
    },
];

/// Initial application inputs, shared by SSR and client rendering.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DemoOptions {
    /// Render every static state rather than the showcase.
    pub fixtures: bool,
    /// Initial scheme.
    pub theme: Theme,
    /// Initial direction.
    pub direction: Direction,
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
    let mut direction = use_signal(|| props.options.direction);
    let mut forest = use_signal(|| false);
    let fixtures = props.options.fixtures;
    rsx! {
        FutureCss { theme: theme(), direction: direction(), brand: forest().then_some(Brand::Forest), stylesheets: props.stylesheets,
            if props.stylesheets == Stylesheets::Automatic { document::Stylesheet { href: assets::COMPONENTS_BUTTON_CSS } }
            div { class: ClassList::new(BG.block()).with(FG.block()).with(PAD.block()).to_string(),
                main { class: ClassList::new(CONTAINER.block()).with(STACK.block()).to_string(),
                    header {
                        p { "FUTURE CSS / DIOXUS" }
                        h1 { if fixtures { "Component fixtures" } else { "A typed foundation for your interface." } }
                        p { "Accessible components. Upstream styles. Rust you can read and run." }
                        nav { aria_label: "Demo pages", class: CLUSTER.block(),
                            a { href: "?fixtures=false", "Showcase" }
                            a { href: "?fixtures=true", "All fixture states" }
                            a { href: "https://github.com/rexlunae/future-css", "Upstream design system" }
                        }
                    }
                    section { "data-showcase": "future-css", aria_labelledby: "appearance-title", class: STACK.block(),
                        h2 { id: "appearance-title", "Make it yours" }
                        p { "Try a scheme, reading direction, or the forest palette. These controls use the same root as your application." }
                        div { class: CLUSTER.block(),
                            label { r#for: "theme", "Color scheme" }
                            select { id: "theme", value: theme().attribute().unwrap_or("system"),
                                onchange: move |event| theme.set(parse_theme(&event.value())),
                                option { value: "system", "System" }
                                option { value: "light", "Light" }
                                option { value: "dark", "Dark" }
                                option { value: "high-contrast-light", "High contrast · light" }
                                option { value: "high-contrast-dark", "High contrast · dark" }
                            }
                            label { r#for: "direction", "Reading direction" }
                            select { id: "direction", value: direction().as_str(),
                                onchange: move |event| direction.set(if event.value() == "rtl" { Direction::Rtl } else { Direction::Ltr }),
                                option { value: "ltr", "Left to right" }
                                option { value: "rtl", "Right to left" }
                            }
                            label {
                                input { r#type: "checkbox", checked: forest(), onchange: move |event| forest.set(event.checked()) }
                                " Forest palette"
                            }
                        }
                        if !fixtures { Source { text: include_str!("examples/future_css/themes.rs"), name: "theme-source" } }
                    }
                    if fixtures {
                        section { "data-fixture": "future-css", aria_label: "Every root scheme",
                            examples::future_css::Themes {}
                        }
                    }
                    section {
                        "data-showcase": "divider", "data-fixture": fixtures.then_some("divider"),
                        aria_labelledby: "divider-title", class: STACK.block(),
                        h2 { id: "divider-title", "Divider" }
                        p { "A quiet break between sections, or a vertical separator within a row." }
                        for example in DIVIDER_EXAMPLES {
                            article { key: "{example.id}", "data-example": example.id, class: STACK.block(),
                                h3 { "{example.title}" }
                                {(example.render)()}
                                if !fixtures { Source { text: example.source, name: example.id } }
                            }
                        }
                    }
                    footer {
                        p { "Initial release · Divider is implemented. The remaining component catalogue is planned in docs/COMPONENTS.md." }
                        p { "Styles are generated from future-css {assets::CSS_VERSION}." }
                    }
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
            button { r#type: "button", class: BUTTON.block(), "data-copy": props.name,
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
    format!("<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>future-css · Dioxus</title><link rel=\"stylesheet\" href=\"/future-css.css\"></head><body>{content}</body></html>")
}

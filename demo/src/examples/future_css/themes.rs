use fc_dioxus::prelude::*;

// example:start
/// Stock visual presets and schemes, using only upstream theme definitions.
#[component]
pub fn Themes() -> Element {
    const BG: Bem = Bem::new("fc-u-bg-canvas");
    const FG: Bem = Bem::new("fc-u-text-default");
    const GRID: Bem = Bem::new("fc-grid-auto");
    rsx! {
        FutureCss { stylesheets: Stylesheets::Inline,
            Card { CardBody {
                p { "Embedded upstream styles for hosts without asset loading." }
                Button { "Styled without stylesheet links" }
            } }
        }
        div { class: GRID.block(),
            for design in Design::ALL {
                FutureCss { design: *design, stylesheets: Stylesheets::External,
                    Card { variant: CardVariant::Feature,
                        h3 { "{design:?}" }
                        p { "A stock future-css visual preset." }
                        Button { variant: ButtonVariant::Primary, "Primary action" }
                    }
                }
            }
        }
        for theme in [Theme::System, Theme::Light, Theme::Dark, Theme::HighContrastLight, Theme::HighContrastDark] {
            FutureCss { theme, stylesheets: Stylesheets::External,
                p { class: ClassList::new(BG.block()).with(FG.block()).to_string(),
                    "{theme:?} theme"
                }
            }
        }
    }
}

use fc_dioxus::prelude::*;

// example:start
/// The five upstream schemes, with no locally authored CSS.
#[component]
pub fn Themes() -> Element {
    const BG: Bem = Bem::new("fc-u-bg-canvas");
    const FG: Bem = Bem::new("fc-u-text-default");
    rsx! {
        for theme in [Theme::System, Theme::Light, Theme::Dark, Theme::HighContrastLight, Theme::HighContrastDark] {
            FutureCss { theme, stylesheets: Stylesheets::External,
                p { class: ClassList::new(BG.block()).with(FG.block()).to_string(),
                    "{theme:?} theme"
                }
            }
        }
    }
}

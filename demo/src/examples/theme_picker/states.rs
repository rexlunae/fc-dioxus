use fc_dioxus::core::platform::{DomBridge, MediaPreference};
use fc_dioxus::prelude::*;

// example:start
/// The picker drives a nested `FutureCss`, so a choice restyles the sample beside it
/// and nothing else on the page.
#[component]
pub fn States() -> Element {
    const STACK: Bem = Bem::new("fc-stack");
    let mut choice = use_signal(|| ThemeChoice::from_design(Design::Helix));
    let mut dark = use_signal(|| false);
    use_hook(move || {
        spawn(async move {
            dark.set(
                DomBridge
                    .prefers(MediaPreference::DarkScheme)
                    .await
                    .unwrap_or(false),
            );
        });
    });
    let picked = choice();
    rsx! {
        div { class: STACK.block(),
            ThemePicker { value: ReadSignal::from(choice), onchange: move |next| choice.set(next) }
            FutureCss { stylesheets: Stylesheets::External, design: picked.design, palette: picked.palette, theme: picked.theme(dark()),
                Card {
                    CardHeader { CardTitle { "Sample" } }
                    CardBody { p { "Styled by the theme chosen above." } }
                    CardFooter { Button { variant: ButtonVariant::Primary, "Continue" } }
                }
            }
        }
    }
}

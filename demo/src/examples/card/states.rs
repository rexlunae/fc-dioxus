use fc_dioxus::prelude::*;

// example:start
/// Linked, elevated, selectable, media, and feature card compositions.
#[component]
pub fn States() -> Element {
    const GRID: Bem = Bem::new("fc-grid-auto");
    const CHECK: Bem = Bem::new("fc-check__input");
    let mut plan = use_signal(|| "team");
    rsx! {
        div { class: GRID.block(),
            Card {
                CardHeader { CardHeading {
                    CardTitle { CardLink { href: "#card", "Research agent" } }
                    CardSubtitle { "Reads the web, cites sources" }
                } }
                CardBody { p { "The whole card is clickable; only the title is the link." } }
                CardFooter { divided: true, Badge { tone: BadgeTone::Success, "Ready" } }
            }
            Card { variant: CardVariant::Elevated, CardBody { p { "Elevated." } } }
            Card { variant: CardVariant::Subtle, CardBody { p { "Subtle." } } }
            Card { as_: CardTag::Label,
                CardHeader { CardHeading { CardTitle { level: None, "Team plan" } }
                    input { r#type: "radio", name: "plan", class: CHECK.block(), checked: plan() == "team", onchange: move |_| plan.set("team") }
                }
            }
            Card { as_: CardTag::Label,
                CardHeader { CardHeading { CardTitle { level: None, "Personal plan" } }
                    input { r#type: "radio", name: "plan", class: CHECK.block(), checked: plan() == "personal", onchange: move |_| plan.set("personal") }
                }
            }
            Card { as_: CardTag::Label,
                CardHeader { CardHeading { CardTitle { level: None, "Enterprise" } CardSubtitle { inline: true, "Contact sales" } }
                    input { r#type: "radio", name: "plan", class: CHECK.block(), disabled: true }
                }
            }
        }
        Card {
            CardLayout { CardMedia { alt: Alt::Decorative }
                div {
                    CardHeader { CardHeading { CardTitle { "Container-aware" } } }
                    CardBody { p { "Media sits beside the text when the card is wide." } }
                }
            }
        }
        Card { variant: CardVariant::Feature,
            h3 { "Theme feature surface" }
            p { "Color and elevation ship with the library." }
        }
    }
}

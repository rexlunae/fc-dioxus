use fc_dioxus::prelude::*;

// example:start
/// Numbered markers in a sentence, and the source list they link to.
#[component]
pub fn States() -> Element {
    let ids = ["src-1", "src-2"].map(Id::try_from);
    let [Ok(plan), Ok(build)] = ids else {
        return rsx! {};
    };
    rsx! {
        p { "Shipped unminified on purpose"
            Cite { index: 1, title: name!("Development plan"), source: plan.clone() }
            ", minified by consumers"
            Cite { index: 2, title: name!("Build orchestrator"), source: build.clone() }
            "."
        }
        Sources {
            Source { id: plan.clone(), index: 1, href: "#{plan}", title: rsx! { "Development plan" }, domain: rsx! { "docs/PLAN.md" } }
            Source { id: build.clone(), index: 2, href: "#{build}", icon: rsx! { "●" },
                title: rsx! { "A source whose title is long enough to wrap onto a second line" }, domain: rsx! { "src/build/build.ts" } }
        }
    }
}

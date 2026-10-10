use fc_dioxus::prelude::*;

const BEFORE: &str = "const a = 1;\nconst b = 2;\nexport { a, b };\n";
const AFTER: &str = "const a = 1;\nconst c = 3;\nexport { a, c }; // a long trailing comment that forces the scrolling mode to scroll\n";

// example:start
/// Added, removed, and unchanged lines with counts; the same diff scrolling sideways;
/// and a longer file whose unchanged stretches fold into an expandable gap.
#[component]
pub fn States() -> Element {
    const STACK: Bem = Bem::new("fc-stack");
    let long_before: String = (1..=30).map(|n| format!("line {n}\n")).collect();
    let long_after = long_before.replace("line 15\n", "line fifteen\n");
    rsx! {
        div { class: STACK.block(),
            DiffView { before: BEFORE, after: AFTER }
            DiffView { before: BEFORE, after: AFTER, scroll: name!("Scrolling diff") }
            DiffView { before: long_before, after: long_after, context: 2 }
        }
    }
}

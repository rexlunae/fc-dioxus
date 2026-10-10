//! A line diff for review, from diff.css, on upstream's `fc-diff` model (Myers' O(ND)
//! diff, the same code upstream can run as WASM).
use crate::{
    core::{
        assets,
        attrs::{component_attributes, diff_attribute},
    },
    models::diff as model,
    prelude::*,
    theme::use_stylesheets,
};
const BEM: Bem = Bem::new("fc-diff");
const STAT: Bem = Bem::new("fc-diff__stat");
fc_modifier! { enum DiffLayout for BEM { Wrap => None, Scroll => "scroll" } }
fc_modifier! { enum StatKind for STAT { Plain => None, Added => "added", Removed => "removed" } }

/// What to draw: a run of lines, or a run of unchanged lines folded away.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Row {
    Line(model::DiffLine),
    Gap {
        from: usize,
        lines: Vec<model::DiffLine>,
    },
}

/// Fold unchanged runs longer than `context` lines on each side of a change.
fn fold(lines: Vec<model::DiffLine>, context: Option<usize>) -> Vec<Row> {
    let Some(context) = context else {
        return lines.into_iter().map(Row::Line).collect();
    };
    let changed: Vec<bool> = lines
        .iter()
        .map(|l| l.kind != model::DiffKind::Equal)
        .collect();
    let near = |i: usize| {
        let low = i.saturating_sub(context);
        let high = (i + context).min(changed.len().saturating_sub(1));
        changed.get(low..=high).is_some_and(|w| w.contains(&true))
    };
    let mut rows = Vec::new();
    let mut pending: Vec<model::DiffLine> = Vec::new();
    let mut pending_from = 0;
    for (index, line) in lines.into_iter().enumerate() {
        if near(index) {
            if !pending.is_empty() {
                rows.push(Row::Gap {
                    from: pending_from,
                    lines: std::mem::take(&mut pending),
                });
            }
            rows.push(Row::Line(line));
        } else {
            if pending.is_empty() {
                pending_from = index;
            }
            pending.push(line);
        }
    }
    if !pending.is_empty() {
        rows.push(Row::Gap {
            from: pending_from,
            lines: pending,
        });
    }
    rows
}

/// The difference between two texts, line by line, with counts above.
///
/// Colour is never the only cue: added and removed lines also differ in their +/−
/// glyph, their accessible label, and `data-fc-diff`. Folded unchanged lines stay in
/// the DOM, hidden, so a machine reading the diff sees all of it.
#[component]
pub fn DiffView(
    before: String,
    after: String,
    /// Keep lines whole and scroll sideways, in a focusable region named by this.
    #[props(default)]
    scroll: Option<AccessibleName>,
    /// Fold unchanged runs, keeping this many lines around each change.
    #[props(default)]
    context: Option<usize>,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let lines = model::diff_lines(&before, &after);
    let stats = model::diff_stats(&lines);
    let rows = fold(lines, context);
    let mut opened = use_signal(Vec::<usize>::new);
    let mut attributes = component_attributes(
        attributes,
        ClassList::new(BEM.block())
            .with_modifier(
                BEM,
                if scroll.is_some() {
                    DiffLayout::Scroll
                } else {
                    DiffLayout::Wrap
                },
            )
            .extend(class.as_deref()),
        &["dir", "role", "aria-label", "tabindex"],
    );
    if let Some(label) = &scroll {
        attributes.push(Attribute::new("role", "region", None, false));
        attributes.push(Attribute::new("aria-label", label.to_string(), None, false));
        attributes.push(Attribute::new("tabindex", "0", None, false));
    }
    let stat = |kind: StatKind| {
        ClassList::new(STAT.block())
            .with_modifier(STAT, kind)
            .to_string()
    };
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_DIFF_CSS } }
        // Source reads left to right in every document.
        div { dir: "ltr", ..attributes,
            div { class: BEM.element("bar"),
                span { class: stat(StatKind::Added), "+{stats.added}" }
                span { class: stat(StatKind::Removed), "−{stats.removed}" }
                if stats.identical { span { class: stat(StatKind::Plain), "No changes" } }
            }
            // Upstream's markup names this `fc-diff__body`, which diff.css does not define.
            div {
                for row in rows {
                    match row {
                        Row::Line(line) => line_view(&line, false),
                        Row::Gap { from, lines } => {
                            let open = opened().contains(&from);
                            let count = lines.len();
                            let plural = if count == 1 { "" } else { "s" };
                            rsx! {
                                if !open {
                                    button { key: "gap-{from}", class: BEM.element("gap"), r#type: "button",
                                        onclick: move |_| opened.write().push(from),
                                        "{count} unchanged line{plural}"
                                    }
                                }
                                for line in lines { {line_view(&line, !open)} }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn line_view(line: &model::DiffLine, hidden: bool) -> Element {
    let (marker, label, data) = match line.kind {
        model::DiffKind::Add => (
            "+",
            Some(format!("Added: {}", line.text)),
            crate::spread![diff_attribute("add")],
        ),
        model::DiffKind::Remove => (
            "−",
            Some(format!("Removed: {}", line.text)),
            crate::spread![diff_attribute("remove")],
        ),
        model::DiffKind::Equal => (" ", None, Spread::default()),
    };
    let old = line.old_line.map(|n| n.to_string()).unwrap_or_default();
    let new = line.new_line.map(|n| n.to_string()).unwrap_or_default();
    rsx! {
        div { class: BEM.element("line"), hidden, aria_label: label, ..data,
            span { class: BEM.element("marker"), aria_hidden: "true", "{marker}" }
            span { class: BEM.element("num"), aria_hidden: "true", "{old}" }
            span { class: BEM.element("num"), aria_hidden: "true", "{new}" }
            span { class: BEM.element("content"), "{line.text}" }
        }
    }
}

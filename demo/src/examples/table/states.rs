use fc_dioxus::prelude::*;

// example:start
/// A sortable, striped table with a sticky header, a selected row, numeric and row-header
/// columns, and an unavailable sort; then loading, empty, and failed tables.
#[component]
pub fn States() -> Element {
    const STACK: Bem = Bem::new("fc-stack");
    let mut selected = use_signal(|| "refactor".to_owned());
    let reason = use_id();
    let columns = vec![
        TableColumn::new("Run").sortable().row_header(),
        TableColumn::new("State").sortable(),
        TableColumn::new("Tokens").sortable().numeric(),
        TableColumn {
            sort: Some(Availability::Unavailable {
                reason: reason.clone(),
            }),
            ..TableColumn::new("Notes")
        },
    ];
    let runs = [
        ("index", "Index docs", "complete", "12,480"),
        ("refactor", "Refactor parser", "running", "88,012"),
        ("triage", "Triage issues", "complete", "9"),
    ];
    let rows: Vec<TableRow> = runs
        .iter()
        .map(|(key, run, state, tokens)| {
            let is_selected = selected() == *key;
            let key_owned = (*key).to_owned();
            TableRow {
                key: (*key).to_owned(),
                selected: is_selected,
                cells: vec![
                    TableCell::text(*run),
                    TableCell::text(*state),
                    TableCell::text(*tokens),
                    TableCell::rich("", rsx! {
                        Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, pressed: is_selected,
                            onclick: move |_| selected.set(key_owned.clone()), "Select" }
                    }),
                ],
            }
        })
        .collect();
    let skeleton = || TableCell::rich("", rsx! { Skeleton { shape: SkeletonShape::Text } });
    rsx! {
        div { class: STACK.block(),
            Table { caption: rsx! { "Runs" }, columns, rows, striped: true, sticky: true,
                default_sort: Some((0, SortDirection::Ascending)) }
            p { id: "{reason}", "Notes cannot be sorted." }
            Table { caption: rsx! { "Loading runs" }, caption_hidden: true, compact: true, busy: true,
                columns: vec![TableColumn::new("Run"), TableColumn::new("Tokens").numeric()],
                rows: vec![TableRow { key: "loading".into(), selected: false, cells: vec![skeleton(), skeleton()] }] }
            Table { caption: rsx! { "No runs" }, caption_hidden: true,
                columns: vec![TableColumn::new("Run"), TableColumn::new("State")], rows: vec![],
                empty: rsx! { "No runs yet" } }
            Table { caption: rsx! { "Runs failed to load" }, caption_hidden: true,
                columns: vec![TableColumn::new("Run"), TableColumn::new("State")], rows: vec![],
                empty: rsx! { EmptyState { compact: true, error: true, body: rsx! { "Could not load runs." } } } }
        }
    }
}

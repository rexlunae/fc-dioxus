use fc_dioxus::models::grid::CellRef;
use fc_dioxus::prelude::*;

// example:start
/// An editable sheet whose edits are stored by the caller; every cell state the CSS
/// draws, with a range selected and a cell mid-edit; and a window of 1,000 rows that
/// still announces its true size.
#[component]
pub fn States() -> Element {
    const STACK: Bem = Bem::new("fc-stack");
    let columns = vec![
        GridColumn::text("Model"),
        GridColumn::text("Provider"),
        GridColumn::numeric("Cost"),
    ];
    let mut sheet = use_signal(|| {
        vec![
            vec!["opus".into(), "anthropic".into(), GridCell::new("18.40")],
            vec!["sonnet".into(), "anthropic".into(), GridCell::new("3.10")],
            vec!["haiku".into(), "anthropic".into(), GridCell::new("0.80")],
        ]
    });
    let mut log = use_signal(String::new);
    let states = vec![
        vec![
            GridCell::new("active"),
            GridCell::new("selected"),
            GridCell::new("18.40"),
        ],
        vec![
            GridCell {
                dirty: true,
                ..GridCell::new("dirty")
            },
            GridCell {
                invalid: true,
                ..GridCell::new("invalid")
            },
            GridCell {
                readonly: true,
                ..GridCell::new("read-only")
            },
        ],
        vec![
            GridCell::new("editing"),
            GridCell::new("rest"),
            GridCell::new("0.00"),
        ],
    ];
    let window: Vec<Vec<GridCell>> = (500..505_u32)
        .map(|n| {
            vec![
                GridCell::new(format!("Row {n}")),
                GridCell::new((n * 3).to_string()),
            ]
        })
        .collect();
    rsx! {
        div { class: STACK.block(),
            DataGrid {
                label: name!("Model costs"),
                columns: columns.clone(),
                rows: sheet,
                onedit: move |edit: GridEdit| {
                    if let Some(cell) = sheet.write().get_mut(edit.cell.row).and_then(|row| row.get_mut(edit.cell.col)) {
                        cell.value.clone_from(&edit.value);
                        cell.dirty = true;
                    }
                    log.set(format!("Edited row {} to {}", edit.cell.row + 1, edit.value));
                },
            }
            p { role: "status", "{log}" }
            DataGrid {
                label: name!("Cell states"),
                columns: vec![GridColumn::text("State"), GridColumn::text("Text"), GridColumn::numeric("Cost")],
                rows: states,
                active: CellRef::new(0, 0),
                anchor: CellRef::new(0, 1),
                editing: CellRef::new(2, 0),
            }
            DataGrid {
                label: name!("A window of a large sheet"),
                columns: vec![GridColumn::text("Name"), GridColumn::numeric("Value")],
                rows: window,
                total_rows: 1000_usize,
                first_row: 499_usize,
            }
        }
    }
}

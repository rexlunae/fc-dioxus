//! A spreadsheet-like data grid from grid.css: sticky headers, one tab stop, rectangular
//! selection, and in-cell editing, on upstream's `fc-grid` cell and range model.
//!
//! It is the UI element, not a spreadsheet: no formulas. Cells are controlled, so an
//! edit is reported through `onedit` and shows once the caller stores it.
use crate::{
    core::{
        assets,
        attrs::{component_attributes, GridData},
        measure::grid_columns,
    },
    models::grid::{CellRange, CellRef},
    prelude::*,
    theme::{use_direction, use_stylesheets, Direction},
};
use std::rc::Rc;
const BEM: Bem = Bem::new("fc-grid");

/// A rows-per-press step for Page Up and Page Down, as upstream's binding uses.
const PAGE_ROWS: usize = 10;

/// One column's header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridColumn {
    /// The header text, which also names the cell editor.
    pub label: String,
    /// Aligns the column to the end with tabular figures.
    pub numeric: bool,
}
impl GridColumn {
    /// A text column.
    #[must_use]
    pub fn text(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            numeric: false,
        }
    }
    /// A numeric column.
    #[must_use]
    pub fn numeric(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            numeric: true,
        }
    }
}

/// One cell's value and state.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GridCell {
    /// The displayed value.
    pub value: String,
    /// Edited and not yet saved: a corner tick.
    pub dirty: bool,
    /// Failed validation.
    pub invalid: bool,
    /// Cannot be edited.
    pub readonly: bool,
}
impl GridCell {
    /// A plain cell.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            ..Self::default()
        }
    }
}
impl<T: Into<String>> From<T> for GridCell {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

/// An edit the user committed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridEdit {
    /// The cell, zero-based within the rendered rows.
    pub cell: CellRef,
    /// The new value.
    pub value: String,
}

/// The data grid. Arrows move, Shift extends the selection, Home and End go to the row
/// ends (with Ctrl, the grid's corners), Page Up and Page Down move ten rows, Ctrl+Space
/// selects the row, and Enter or F2 edits. Tab leaves the grid.
#[component]
pub fn DataGrid(
    label: AccessibleName,
    columns: Vec<GridColumn>,
    rows: Vec<Vec<GridCell>>,
    /// Numbers each row in a sticky header column.
    #[props(default = true)]
    row_headers: bool,
    /// The true row count when only a window of rows is rendered, so a screen reader
    /// is told the real size.
    #[props(default)]
    total_rows: Option<usize>,
    /// The zero-based index of the first rendered row within `total_rows`.
    #[props(default)]
    first_row: usize,
    /// The cell active on first render.
    #[props(default = CellRef::new(0, 0))]
    active: CellRef,
    /// Where the first render's selection starts; the active cell when omitted.
    #[props(default)]
    anchor: Option<CellRef>,
    /// A cell that starts in edit mode, such as the first cell of a new row.
    #[props(default)]
    editing: Option<CellRef>,
    #[props(default)] onedit: Option<EventHandler<GridEdit>>,
    #[props(default)] onselect: Option<EventHandler<CellRange>>,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let width = columns.len();
    let height = rows.len();
    let clamp = move |cell: CellRef| {
        CellRef::new(
            cell.row.min(height.saturating_sub(1)),
            cell.col.min(width.saturating_sub(1)),
        )
    };
    let mut current = use_signal(|| clamp(active));
    let mut origin = use_signal(|| clamp(anchor.unwrap_or(active)));
    let mut edit = use_signal(|| editing.map(clamp));
    let mut draft = use_signal(String::new);
    // Only an edit the user starts takes focus; one present on first render must not
    // pull focus from wherever the page put it.
    let mut claim = use_signal(|| false);
    let mut nodes = use_signal(Vec::<(CellRef, Rc<MountedData>)>::new);
    let direction = use_direction();
    let range = CellRange::from_corners(origin(), current());

    let focus = move |cell: CellRef| {
        let node = nodes
            .peek()
            .iter()
            .find(|(at, _)| *at == cell)
            .map(|(_, m)| m.clone());
        if let Some(node) = node {
            spawn(async move {
                let _ = node.set_focus(true).await;
            });
        }
    };
    let mut go = move |cell: CellRef, extend: bool| {
        let cell = clamp(cell);
        if !extend {
            origin.set(cell);
        }
        current.set(cell);
        if let Some(onselect) = onselect {
            onselect.call(CellRange::from_corners(origin(), cell));
        }
        focus(cell);
    };
    // The latest cells, readable from handlers without re-rendering anything.
    let mut cells = use_hook(|| CopyValue::new(rows.clone()));
    if *cells.peek() != rows {
        cells.set(rows.clone());
    }
    let cell_at = move |cell: CellRef| {
        cells
            .peek()
            .get(cell.row)
            .and_then(|r| r.get(cell.col))
            .cloned()
    };
    let mut begin = move |cell: CellRef| {
        if let Some(found) = cell_at(cell).filter(|c| !c.readonly) {
            draft.set(found.value);
            claim.set(true);
            edit.set(Some(cell));
        }
    };
    // A cell that starts in edit mode starts from its value.
    use_hook(|| {
        if let Some(found) = editing.map(clamp).and_then(cell_at) {
            draft.set(found.value);
        }
    });
    // `refocus` is false when focus left the editor on its own: it went somewhere the
    // user chose, and taking it back would be theft.
    let mut finish = move |commit: bool, refocus: bool| {
        let Some(cell) = edit() else { return };
        edit.set(None);
        if commit {
            if let Some(onedit) = onedit {
                onedit.call(GridEdit {
                    cell,
                    value: draft(),
                });
            }
        }
        if refocus {
            focus(cell);
        }
    };

    let mut attributes = component_attributes(
        attributes,
        ClassList::new(BEM.block()).extend(class.as_deref()),
        &[],
    );
    attributes.push(grid_columns(row_headers, width));
    let total = total_rows.unwrap_or(height);
    attributes.push(GridData::RowCount.attribute(&total));
    attributes.push(GridData::ColCount.attribute(&width));
    // Declared only when rows are missing from the DOM: then the count is the only way
    // to learn that row 40 is not the end. It counts the header row.
    let virtual_rows = total > height;
    let ctrl = |event: &KeyboardEvent| event.modifiers().ctrl() || event.modifiers().meta();

    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_GRID_CSS } }
        div { role: "grid", aria_label: "{label}", aria_multiselectable: "true",
            aria_rowcount: if virtual_rows { Some((total + 1).to_string()) } else { None },
            onkeydown: move |event| {
                if edit.peek().is_some() { return; }
                let at = current();
                let extend = event.modifiers().shift();
                let (back, forward) = if direction == Direction::Rtl { (Key::ArrowRight, Key::ArrowLeft) } else { (Key::ArrowLeft, Key::ArrowRight) };
                let key = event.key();
                let target = if key == Key::ArrowUp {
                    Some(CellRef::new(at.row.saturating_sub(1), at.col))
                } else if key == Key::ArrowDown {
                    Some(CellRef::new(at.row + 1, at.col))
                } else if key == back {
                    Some(CellRef::new(at.row, at.col.saturating_sub(1)))
                } else if key == forward {
                    Some(CellRef::new(at.row, at.col + 1))
                } else if key == Key::Home {
                    Some(if ctrl(&event) { CellRef::new(0, 0) } else { CellRef::new(at.row, 0) })
                } else if key == Key::End {
                    Some(if ctrl(&event) { CellRef::new(usize::MAX, usize::MAX) } else { CellRef::new(at.row, usize::MAX) })
                } else if key == Key::PageUp {
                    Some(CellRef::new(at.row.saturating_sub(PAGE_ROWS), at.col))
                } else if key == Key::PageDown {
                    Some(CellRef::new(at.row + PAGE_ROWS, at.col))
                } else {
                    None
                };
                if let Some(target) = target {
                    event.prevent_default();
                    go(target, extend);
                } else if key == Key::Enter || key == Key::F2 {
                    event.prevent_default();
                    begin(at);
                } else if key == Key::Character(" ".into()) && ctrl(&event) {
                    // Ctrl+Space selects the row; a bare space is left to scroll.
                    event.prevent_default();
                    origin.set(CellRef::new(at.row, 0));
                    go(CellRef::new(at.row, usize::MAX), true);
                }
            },
            ..attributes,
            div { role: "row", aria_rowindex: if virtual_rows { Some("1") } else { None },
                if row_headers { span { class: BEM.element("corner"), role: "columnheader" } }
                for column in columns.iter() {
                    span { class: ClassList::new(&BEM.element("col-header")).with_if(column.numeric, BEM.element("col-header--numeric")).to_string(), role: "columnheader",
                        "{column.label}"
                    }
                }
            }
            for (r, row) in rows.iter().enumerate() {
                div { key: "{first_row + r}", role: "row",
                    aria_rowindex: if virtual_rows { Some((first_row + r + 2).to_string()) } else { None },
                    if row_headers { span { class: BEM.element("row-header"), role: "rowheader", "{first_row + r + 1}" } }
                    for (c, cell) in row.iter().enumerate() {
                        {
                            let at = CellRef::new(r, c);
                            let numeric = columns.get(c).is_some_and(|col| col.numeric);
                            let editing = edit() == Some(at);
                            let is_active = current() == at;
                            let name = format!("{} {}", columns.get(c).map_or("", |col| col.label.as_str()), first_row + r + 1);
                            let mut data = Spread::default();
                            if is_active { data.push(GridData::Active.attribute("true")); }
                            if cell.dirty { data.push(GridData::Dirty.attribute("true")); }
                            let classes = ClassList::new(&BEM.element("cell"))
                                .with_if(numeric, BEM.element("cell--numeric"))
                                .with_if(editing, BEM.element("cell--editing"))
                                .to_string();
                            rsx! {
                                span { key: "{c}", class: classes, role: "gridcell",
                                    // Kept while editing: focus returns here before the editor unmounts, and
                                    // a span without a tabindex cannot take it.
                                    tabindex: if is_active { "0" } else { "-1" },
                                    aria_selected: range.contains(at).to_string(),
                                    aria_invalid: if cell.invalid { Some("true") } else { None },
                                    aria_readonly: if cell.readonly { Some("true") } else { None },
                                    onmounted: move |event: MountedEvent| {
                                        let mut nodes = nodes.write();
                                        nodes.retain(|(cell, _)| *cell != at);
                                        nodes.push((at, event.data()));
                                    },
                                    onclick: move |event: MouseEvent| {
                                        let extend = event.modifiers().shift() || event.modifiers().ctrl() || event.modifiers().meta();
                                        go(at, extend);
                                    },
                                    ondoubleclick: move |_| begin(at),
                                    ..data,
                                    if editing {
                                        input { class: BEM.element("input"), r#type: "text", aria_label: "{name}", value: "{draft}",
                                            onmounted: move |event: MountedEvent| async move {
                                                if claim() {
                                                    let _ = event.data().set_focus(true).await;
                                                }
                                            },
                                            oninput: move |event| draft.set(event.value()),
                                            onblur: move |_| finish(true, false),
                                            onkeydown: move |event: KeyboardEvent| {
                                                // Arrows stay with the text: the grid's keys would
                                                // trap the user in the editor or move under them.
                                                event.stop_propagation();
                                                if event.key() == Key::Enter {
                                                    event.prevent_default();
                                                    finish(true, true);
                                                } else if event.key() == Key::Escape {
                                                    // A half-typed value is discarded, as in
                                                    // every spreadsheet.
                                                    event.prevent_default();
                                                    finish(false, true);
                                                }
                                            },
                                        }
                                    } else {
                                        "{cell.value}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

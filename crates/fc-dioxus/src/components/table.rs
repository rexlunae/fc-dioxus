//! Data tables with sortable columns, from table.css. Sorting ports upstream's
//! `table.ts`; see [`crate::behavior::sort`].
use crate::{
    behavior::sort::{self, SortDirection},
    core::{
        assets,
        attrs::{availability_attributes, component_attributes},
    },
    prelude::*,
    theme::use_stylesheets,
};
const BEM: Bem = Bem::new("fc-table");
const WRAP: Bem = Bem::new("fc-table-wrap");
const NUM: Bem = Bem::new("fc-table__cell--num");
const HIDDEN: Bem = Bem::new("fc-u-visually-hidden");

/// One column.
#[derive(Debug, Clone, PartialEq)]
pub struct TableColumn {
    /// Header text.
    pub header: String,
    /// Right-align figures so they compare at a glance.
    pub numeric: bool,
    /// Cells in this column are row headers (`<th scope="row">`).
    pub row_header: bool,
    /// Whether the header offers sorting, and whether it can be used now.
    pub sort: Option<Availability>,
}
impl TableColumn {
    /// A plain, unsortable column.
    #[must_use]
    pub fn new(header: impl Into<String>) -> Self {
        Self {
            header: header.into(),
            numeric: false,
            row_header: false,
            sort: None,
        }
    }
    /// A column that can be sorted.
    #[must_use]
    pub fn sortable(mut self) -> Self {
        self.sort = Some(Availability::Enabled);
        self
    }
    /// Right-aligned figures.
    #[must_use]
    pub fn numeric(mut self) -> Self {
        self.numeric = true;
        self
    }
    /// Cells are row headers.
    #[must_use]
    pub fn row_header(mut self) -> Self {
        self.row_header = true;
        self
    }
}

/// One cell: what sorts, and optionally richer content to show.
#[derive(Debug, Clone, PartialEq)]
pub struct TableCell {
    /// The value sorting compares. What is shown is not always what sorts: "3 min ago"
    /// or "12.5 KB" need their raw value here.
    pub sort_key: String,
    /// Shown instead of `sort_key` when given.
    pub content: Option<Element>,
}
impl TableCell {
    /// A cell showing and sorting by the same text.
    #[must_use]
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            sort_key: text.into(),
            content: None,
        }
    }
    /// A cell that shows `content` and sorts by `sort_key`.
    #[must_use]
    pub fn rich(sort_key: impl Into<String>, content: Element) -> Self {
        Self {
            sort_key: sort_key.into(),
            content: Some(content),
        }
    }
}

/// One row.
#[derive(Debug, Clone, PartialEq)]
pub struct TableRow {
    /// Stable identity, so a row keeps its state when sorting moves it.
    pub key: String,
    /// Cells, one per column.
    pub cells: Vec<TableCell>,
    /// The row is selected (`aria-selected`).
    pub selected: bool,
}

/// Who reorders the rows when the sort changes.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SortMode {
    /// The table sorts its rows.
    #[default]
    Client,
    /// Only `aria-sort` moves; `onsort` asks the owner for new rows, e.g. from a server.
    Manual,
}

fc_modifier! { enum Density for BEM { Comfortable => None, Compact => "compact" } }
fc_modifier! { enum Bands for BEM { Plain => None, Striped => "striped" } }
fc_modifier! { enum Header for BEM { Scrolls => None, Sticky => "sticky" } }

/// A table in a focusable, named scroll region, with optional sorting.
///
/// Only the sorted column carries `aria-sort`, so screen readers do not announce "not
/// sorted" on every other header; the arrow and emphasis come from the same attribute.
#[component]
pub fn Table(
    /// The caption, which also names the scroll region.
    caption: Element,
    /// Hide the caption visually; it still names the table and region.
    #[props(default)]
    caption_hidden: bool,
    columns: Vec<TableColumn>,
    rows: Vec<TableRow>,
    #[props(default)] sort: Option<ReadSignal<Option<(usize, SortDirection)>>>,
    #[props(default)] default_sort: Option<(usize, SortDirection)>,
    #[props(default)] onsort: Option<EventHandler<(usize, SortDirection)>>,
    #[props(default)] mode: SortMode,
    #[props(default)] striped: bool,
    #[props(default)] compact: bool,
    /// Keep the header in view while the body scrolls.
    #[props(default)]
    sticky: bool,
    /// Rows are loading.
    #[props(default)]
    busy: bool,
    /// Shown across the table when there are no rows.
    #[props(default)]
    empty: Option<Element>,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let state = use_controllable(
        sort,
        || default_sort,
        onsort.map(|onsort| {
            EventHandler::new(move |value: Option<(usize, SortDirection)>| {
                if let Some(value) = value {
                    onsort.call(value);
                }
            })
        }),
    );
    let id = use_id();
    let caption_id = id.suffixed("caption");
    let current = state.get();
    let order: Vec<usize> = match (mode, current) {
        (SortMode::Client, Some((column, direction))) => {
            let keys: Vec<&str> = rows
                .iter()
                .map(|row| row.cells.get(column).map_or("", |c| c.sort_key.as_str()))
                .collect();
            sort::order(&keys, direction)
        }
        _ => (0..rows.len()).collect(),
    };
    let attributes = component_attributes(
        attributes,
        ClassList::new(BEM.block())
            .with_modifier(
                BEM,
                if compact {
                    Density::Compact
                } else {
                    Density::Comfortable
                },
            )
            .with_modifier(
                BEM,
                if striped {
                    Bands::Striped
                } else {
                    Bands::Plain
                },
            )
            .with_modifier(
                BEM,
                if sticky {
                    Header::Sticky
                } else {
                    Header::Scrolls
                },
            )
            .extend(class.as_deref()),
        &["aria-busy"],
    );
    let span = columns.len().max(1).to_string();
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_TABLE_CSS } }
        div { class: WRAP.block(), role: "region", aria_labelledby: "{caption_id}", tabindex: "0",
            table { aria_busy: busy.then_some("true"), ..attributes,
                caption { id: "{caption_id}", class: caption_hidden.then(|| HIDDEN.block()), {caption} }
                thead { tr {
                    for (index, column) in columns.iter().enumerate() {
                        th { key: "{index}", scope: "col", class: column.numeric.then(|| NUM.block()),
                            aria_sort: current.filter(|(sorted, _)| *sorted == index).map(|(_, d)| d.as_str()),
                            match &column.sort {
                                Some(availability) => {
                                    let enabled = *availability == Availability::Enabled;
                                    let blocked = availability_attributes(availability);
                                    rsx! {
                                        button { class: BEM.element("sort"), r#type: "button",
                                            onclick: move |_| if enabled { state.set(Some((index, sort::next_direction(state.get(), index)))); },
                                            ..blocked, "{column.header}"
                                        }
                                    }
                                }
                                None => rsx! { "{column.header}" },
                            }
                        }
                    }
                } }
                tbody {
                    if rows.is_empty() {
                        if let Some(empty) = empty { tr { td { class: BEM.element("empty"), colspan: "{span}", {empty} } } }
                    }
                    for index in order {
                        if let Some(row) = rows.get(index) {
                            tr { key: "{row.key}", aria_selected: row.selected.then_some("true"),
                                for (column_index, cell) in row.cells.iter().enumerate() {
                                    {cell_view(columns.get(column_index), cell)}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn cell_view(column: Option<&TableColumn>, cell: &TableCell) -> Element {
    let numeric = column.is_some_and(|c| c.numeric).then(|| NUM.block());
    let content = cell
        .content
        .clone()
        .unwrap_or_else(|| rsx! { "{cell.sort_key}" });
    if column.is_some_and(|c| c.row_header) {
        rsx! { th { scope: "row", class: numeric, {content} } }
    } else {
        rsx! { td { class: numeric, {content} } }
    }
}

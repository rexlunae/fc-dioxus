//! A working-tree view: branch, status counts, changed files, and per-file line stats,
//! from git.css, on upstream's `fc-git` model.
//!
//! It runs no Git and parses no porcelain: it is a view over status entries the
//! application already has, from libgit2, a binding, or anywhere else.
use crate::{
    core::{
        assets,
        attrs::{component_attributes, GitData},
    },
    models::git as model,
    prelude::*,
    theme::use_stylesheets,
};
use std::rc::Rc;
const BEM: Bem = Bem::new("fc-git");

/// Where a file is, on one side (index or worktree).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum FileState {
    /// New.
    Added,
    /// Changed.
    Modified,
    /// Removed.
    Deleted,
    /// Moved.
    Renamed,
    /// Not tracked yet.
    Untracked,
    /// Merge conflict.
    Conflicted,
}
impl FileState {
    const fn code(self) -> u8 {
        match self {
            Self::Added => 0,
            Self::Modified => 1,
            Self::Deleted => 2,
            Self::Renamed => 3,
            Self::Untracked => 4,
            Self::Conflicted => 5,
        }
    }
    /// The state's name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Added => "added",
            Self::Modified => "modified",
            Self::Deleted => "deleted",
            Self::Renamed => "renamed",
            Self::Untracked => "untracked",
            Self::Conflicted => "conflicted",
        }
    }
    /// The glyph drawn under forced colors, where the badge colour is discarded.
    const fn glyph(self) -> &'static str {
        match self {
            Self::Added => "+",
            Self::Modified => "~",
            Self::Deleted => "−",
            Self::Renamed => ">",
            Self::Untracked => "?",
            Self::Conflicted => "!",
        }
    }
}

/// One changed file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitEntry {
    /// Repository-relative path.
    pub path: String,
    /// State in the index (staged), if any.
    pub index: Option<FileState>,
    /// State in the worktree (not staged), if any.
    pub worktree: Option<FileState>,
    /// Contents before, when loaded; with `after`, gives the line stat.
    pub before: Option<String>,
    /// Contents after, when loaded.
    pub after: Option<String>,
}
impl GitEntry {
    fn conflicted(&self) -> bool {
        self.index == Some(FileState::Conflicted) || self.worktree == Some(FileState::Conflicted)
    }
    fn modeled(&self) -> model::Entry {
        model::Entry {
            index: self.index.map_or(255, FileState::code),
            worktree: self.worktree.map_or(255, FileState::code),
        }
    }
}

/// "1 conflicted · 2 staged · 3 not staged · 1 untracked", or "Working tree clean".
#[must_use]
pub fn summary(entries: &[GitEntry]) -> String {
    let counts =
        model::summarize_status(&entries.iter().map(GitEntry::modeled).collect::<Vec<_>>());
    let mut parts = Vec::new();
    for (count, label) in [
        (counts.conflicted, "conflicted"),
        (counts.staged, "staged"),
        (counts.unstaged, "not staged"),
        (counts.untracked, "untracked"),
    ] {
        if count > 0 {
            parts.push(format!("{count} {label}"));
        }
    }
    if parts.is_empty() {
        "Working tree clean".to_owned()
    } else {
        parts.join(" · ")
    }
}

/// Entries in the order that needs attention first: conflicts, then files changed on
/// both sides, then the rest, each group by path.
#[must_use]
pub fn ordered(entries: &[GitEntry]) -> Vec<GitEntry> {
    // fc-git sorts by rank only and requires path order going in.
    let mut by_path = entries.to_vec();
    by_path.sort_by(|a, b| a.path.cmp(&b.path));
    let modeled: Vec<model::Entry> = by_path.iter().map(GitEntry::modeled).collect();
    model::order_entries(&modeled)
        .into_iter()
        .filter_map(|i| by_path.get(usize::try_from(i).ok()?).cloned())
        .collect()
}

/// The branch, the status counts, and the changed files as a grid with one tab stop.
/// Arrows, Home, and End move the selection; a Stage button sits on each unstaged file.
#[component]
pub fn GitSurface(
    /// The branch, or `None` before the first commit.
    branch: Option<String>,
    entries: Vec<GitEntry>,
    /// Names the file grid; "Changed files" when omitted.
    #[props(default)]
    label: Option<AccessibleName>,
    #[props(default)] onselect: Option<EventHandler<String>>,
    #[props(default)] onstage: Option<EventHandler<String>>,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let rows = ordered(&entries);
    let first = rows.first().map(|e| e.path.clone());
    let selected = use_signal(|| first);
    let mounted = use_signal(Vec::<(String, Rc<MountedData>)>::new);
    let base = use_id();
    let counts =
        model::summarize_status(&entries.iter().map(GitEntry::modeled).collect::<Vec<_>>());
    let mut attributes = component_attributes(
        attributes,
        ClassList::new(BEM.block()).extend(class.as_deref()),
        &[],
    );
    attributes.extend([
        GitData::Branch.attribute(&branch.clone().unwrap_or_default()),
        GitData::BranchState.attribute(if branch.is_some() { "named" } else { "unborn" }),
        GitData::Staged.attribute(&counts.staged),
        GitData::Unstaged.attribute(&counts.unstaged),
        GitData::Conflicted.attribute(&counts.conflicted),
        GitData::Untracked.attribute(&counts.untracked),
    ]);
    if rows.is_empty() {
        attributes.push(GitData::Empty.attribute(&"true"));
    }
    let base = base.to_string();
    let label = label.map_or_else(|| "Changed files".to_owned(), |l| l.to_string());
    let empty = rows.is_empty();
    let paths: Vec<String> = rows.iter().map(|e| e.path.clone()).collect();
    rsx! {
        if use_stylesheets() {
            // grid.css styles every `[role="row"]` as `display: contents`, and git rows
            // are rows. Upstream's bundle puts grid before git, so git's own row rule
            // wins on equal specificity; per-component links load in mount order, so a
            // grid mounted after this surface would collapse its rows and make them
            // unfocusable. Links dedupe by href, so asking for grid.css first here pins
            // the bundle order whichever mounts first.
            document::Stylesheet { href: assets::COMPONENTS_GRID_CSS }
            document::Stylesheet { href: assets::COMPONENTS_GIT_CSS }
        }
        div { ..attributes,
            div { class: BEM.element("header"),
                // A repository with no commits has a branch and no history.
                span { class: BEM.element("branch"), "{branch.clone().unwrap_or_else(|| \"Initial commit\".to_owned())}" }
                span { class: BEM.element("summary"), "{summary(&entries)}" }
            }
            div { class: BEM.element("files"), role: if empty { "group" } else { "grid" }, aria_label: "{label}",
                if empty {
                    p { class: BEM.element("empty"), "No changes" }
                }
                for (index, entry) in rows.into_iter().enumerate() {
                    {row_view(&entry, index, &base, selected, mounted, paths.clone(), onselect, onstage)}
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn row_view(
    entry: &GitEntry,
    index: usize,
    base: &str,
    mut selected: Signal<Option<String>>,
    mut mounted: Signal<Vec<(String, Rc<MountedData>)>>,
    paths: Vec<String>,
    onselect: Option<EventHandler<String>>,
    onstage: Option<EventHandler<String>>,
) -> Element {
    let base = base.to_owned();
    let id = row_id(&base, index);
    let on = selected().as_deref() == Some(entry.path.as_str());
    let (dir, base) = entry
        .path
        .rsplit_once('/')
        .map_or((String::new(), entry.path.clone()), |(d, b)| {
            (format!("{d}/"), b.to_owned())
        });
    let conflicted = entry.conflicted();
    let stat = (entry.before.is_some() || entry.after.is_some())
        .then(|| model::diff_counts(entry.before.as_deref(), entry.after.as_deref()))
        .filter(|(added, removed)| *added > 0 || *removed > 0);
    let path = entry.path.clone();
    let mut choose = move |path: String| {
        selected.set(Some(path.clone()));
        if let Some(onselect) = onselect {
            onselect.call(path);
        }
    };
    let data = crate::spread![
        GitData::Path.attribute(&entry.path),
        GitData::Selected.attribute(&on),
    ];
    rsx! {
        div { key: "{entry.path}", class: BEM.element("row"), id: "{id}", role: "row", aria_selected: on.to_string(),
            tabindex: if on { "0" } else { "-1" },
            onclick: { let path = path.clone(); move |_| choose(path.clone()) },
            onmounted: { let path = path.clone(); move |event: MountedEvent| {
                let mut nodes = mounted.write();
                nodes.retain(|(p, _)| *p != path);
                nodes.push((path.clone(), event.data()));
            } },
            onkeydown: move |event| {
                let last = paths.len().saturating_sub(1);
                let next = match event.key() {
                    Key::ArrowDown => Some((index + 1).min(last)),
                    Key::ArrowUp => Some(index.saturating_sub(1)),
                    Key::Home => Some(0),
                    Key::End => Some(last),
                    _ => None,
                };
                if let Some(next) = next {
                    event.prevent_default();
                    if let Some(target) = paths.get(next) {
                        choose(target.clone());
                        let node = mounted.peek().iter().find(|(p, _)| p == target).map(|(_, m)| m.clone());
                        if let Some(node) = node {
                            spawn(async move { let _ = node.set_focus(true).await; });
                        }
                    }
                } else if matches!(event.key(), Key::Enter) || event.key() == Key::Character(" ".into()) {
                    event.prevent_default();
                    choose(path.clone());
                }
            },
            ..data,
            span { class: BEM.element("file-name"), role: "gridcell",
                if !dir.is_empty() { span { class: BEM.element("file-dir"), "{dir}" } }
                "{base}"
            }
            span { class: BEM.element("stages"), role: "gridcell",
                if let Some(state) = entry.index { {badge("index", state, conflicted)} }
                if let Some(state) = entry.worktree { {badge("worktree", state, conflicted)} }
                if entry.index.is_none() && !conflicted {
                    button { r#type: "button", class: BEM.element("stage-button"), aria_label: "Stage {entry.path}",
                        onclick: { let path = entry.path.clone(); move |event: MouseEvent| { event.stop_propagation(); if let Some(onstage) = onstage { onstage.call(path.clone()); } } },
                        // The row's keys would cancel the button's own Enter and Space.
                        onkeydown: move |event: KeyboardEvent| event.stop_propagation(),
                        ..crate::spread![GitData::StagePath.attribute(&entry.path)],
                        "Stage"
                    }
                }
            }
            span { class: BEM.element("stat"), role: "gridcell", hidden: stat.is_none(),
                if let Some((added, removed)) = stat {
                    span { class: BEM.element("stat-add"), "+{added}" }
                    span { class: BEM.element("stat-remove"), "−{removed}" }
                }
            }
        }
    }
}

fn badge(side: &'static str, state: FileState, conflicted: bool) -> Element {
    let shown = if conflicted {
        FileState::Conflicted
    } else {
        state
    };
    let place = if side == "index" {
        "staged"
    } else {
        "not staged"
    };
    // The side is in the title too: "modified" alone does not say whether it is staged.
    let title = format!("{} ({place})", shown.as_str());
    let data = crate::spread![
        GitData::Stage.attribute(&side),
        GitData::State.attribute(&shown.as_str()),
        GitData::Glyph.attribute(&shown.glyph()),
    ];
    rsx! { span { class: BEM.element("badge"), title, ..data, "{state.as_str()}" } }
}

fn row_id(base: &str, index: usize) -> String {
    format!("{base}-row-{index}")
}

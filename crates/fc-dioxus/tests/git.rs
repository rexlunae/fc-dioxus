//! Git surface status text and ordering, against the behaviour upstream's git.ts documents.
#![forbid(unsafe_code)]

use fc_dioxus::components::git::{ordered, summary};
use fc_dioxus::prelude::{FileState, GitEntry};

fn entry(path: &str, index: Option<FileState>, worktree: Option<FileState>) -> GitEntry {
    GitEntry {
        path: path.to_owned(),
        index,
        worktree,
        before: None,
        after: None,
    }
}

/// A file changed on both sides is in both counts, and untracked files are not
/// staged, so they count as not staged too.
#[test]
fn a_clean_tree_says_so_and_counts_join_in_attention_order() {
    assert_eq!(summary(&[]), "Working tree clean");
    let tree = [
        entry("a", Some(FileState::Modified), Some(FileState::Modified)),
        entry("b", None, Some(FileState::Untracked)),
        entry(
            "c",
            Some(FileState::Conflicted),
            Some(FileState::Conflicted),
        ),
        entry("d", Some(FileState::Added), None),
    ];
    assert_eq!(
        summary(&tree),
        "1 conflicted · 3 staged · 3 not staged · 1 untracked"
    );
}

#[test]
fn conflicts_come_first_then_files_changed_on_both_sides_then_paths() {
    let tree = [
        entry("z.rs", Some(FileState::Added), None),
        entry("a.rs", None, Some(FileState::Modified)),
        entry("m.rs", Some(FileState::Modified), Some(FileState::Modified)),
        entry(
            "q.rs",
            Some(FileState::Conflicted),
            Some(FileState::Conflicted),
        ),
    ];
    let paths: Vec<String> = ordered(&tree).into_iter().map(|e| e.path).collect();
    // Within a rank, path order, whatever order they arrived in.
    assert_eq!(paths, ["q.rs", "m.rs", "a.rs", "z.rs"]);
}

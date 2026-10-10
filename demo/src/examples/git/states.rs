use fc_dioxus::prelude::*;

fn entry(path: &str, index: Option<FileState>, worktree: Option<FileState>) -> GitEntry {
    GitEntry {
        path: path.to_owned(),
        index,
        worktree,
        before: None,
        after: None,
    }
}

// example:start
/// A tree with every file state, a conflict sorted first, and line stats; a clean
/// tree; and a repository with no commits yet.
#[component]
pub fn States() -> Element {
    const STACK: Bem = Bem::new("fc-stack");
    use FileState::{Added, Conflicted, Deleted, Modified, Renamed, Untracked};
    let mut log = use_signal(String::new);
    let tree = vec![
        GitEntry {
            before: Some("a\nb\nc\n".to_owned()),
            after: Some("a\nc\nd\nd\n".to_owned()),
            ..entry("src/js/git.ts", Some(Modified), Some(Modified))
        },
        GitEntry {
            after: Some("one\ntwo\n".to_owned()),
            ..entry("crates/fc-git/src/lib.rs", None, Some(Untracked))
        },
        entry("src/js/merge.ts", Some(Conflicted), Some(Conflicted)),
        entry("README.md", Some(Added), None),
        entry("src/old-name.ts", Some(Deleted), None),
        entry("src/new-name.ts", Some(Renamed), None),
        entry(
            "packages/a/deeply/nested/directory/structure/that/keeps/going/component.tsx",
            None,
            Some(Modified),
        ),
    ];
    let unborn = tree.iter().take(2).cloned().collect::<Vec<_>>();
    rsx! {
        div { class: STACK.block(),
            GitSurface {
                branch: "feat/a-branch-name-long-enough-to-truncate".to_owned(),
                entries: tree,
                onselect: move |path| log.set(format!("Selected {path}")),
                onstage: move |path| log.set(format!("Stage requested for {path}")),
            }
            p { role: "status", "{log}" }
            GitSurface { branch: "main".to_owned(), entries: Vec::new() }
            GitSurface { branch: None, entries: unborn }
        }
    }
}

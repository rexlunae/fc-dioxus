# fc-dioxus development plan

Phases with exit gates. As in upstream's
[PLAN.md](https://github.com/rexlunae/future-css/blob/main/docs/PLAN.md), each gate can be
checked, not judged. Every phase ends with `cargo xtask check` green.

**Current status: initial workspace built, including the Phase 0 foundations.**

Implemented: all five workspace packages, a pinned/reproducible upstream sync,
workspace safety lints, renderer build commands, a CI gate, generated token names
and scheme values, and the first component (`Divider`) with compiled demo snippets,
fixtures, structural parity, coverage accounting, and browser checks.

The initial core includes classes/modifiers, validated names and IDs, deterministic
ID allocation, typed state attributes, controlled state, root themes, and an async
host bridge. `fc-a11y` has four tested static rules and a human-readable CLI.
These are useful slices of Phases 1 and 2, not completion of those phases. Remaining
core types/hooks, full compliance tooling, all other wrappers, and renderer runtime
smokes remain in their phases below.

Upstream [#43](https://github.com/rexlunae/future-css/issues/43) is resolved by merged
[PR #44](https://github.com/rexlunae/future-css/pull/44). All eight model crates are
pinned dependencies with default features disabled; the gate checks that their
dependency graphs contain no bindgen glue. An upstream fixture ID collision found
by the new audit is fixed in
[PR #46](https://github.com/rexlunae/future-css/pull/46), whose commit is currently
pinned. No generated fixture was patched locally.

| Phase | Delivers | Gate |
| --- | --- | --- |
| 0. Foundations | Repository, upstream pin and sync, lints, model crates gated upstream, demo skeleton on every renderer | Workspace builds for native and wasm32; the empty demo builds for every renderer |
| 1. Core and demo shell | Shared types, macro, ids, controlled state, platform bridges, theme root, test harness, showcase and fixture pages | `Divider` complete: in the demo with its Rust source, parity and demo tests green |
| 2. Compliance engine | `fc-a11y` rules, HTML adapter, CLI `check`, `fc-tokens` | Audit of upstream's `fixtures.html` and the demo's fixture page with zero errors |
| 3. Static components | Every component with no keyboard behaviour, each in the demo | Parity, a11y, coverage, and demo tests green for each |
| 4. Interactive components | Shared behaviours and every bound component, each in the demo | Browser suite with keyboard scripts and axe-core, on web and liveview |
| 5. Heavy components | Calendar, highlighting, terminal, data grid, webview, editor | Their upstream crates shipped and pinned; browser suite |
| 6. Compliance tooling and renderers | Debug checks, overlay, ledger, conformance report, SARIF, renderer support matrix | An example app audited end to end; `RENDERERS.md` generated from real runs |
| 7. Release | Names, publishing, hosted demo | Published crates, docs.rs builds, demo online |

**Upstream work runs alongside.** Each phase files the upstream issues it needs when it
starts, with a concrete API, and lists them in
[COMPONENTS.md, Waiting on upstream](COMPONENTS.md#waiting-on-upstream). Work that waits on
upstream moves to the end of its phase rather than blocking the rest; nothing is built here
in its place.

---

## Phase 0 -- Foundations

1. **Create the repository** with a Cargo workspace: `crates/fc-dioxus`, `crates/fc-a11y`,
   `crates/fc-tokens`, `demo/` (`fc-demo`, unpublished), and `xtask/`.
2. **Verify the toolchain before writing code.** Resolve Dioxus 0.7 together with the
   upstream model crates as git dependencies. Upstream pins `wasm-bindgen = "=0.2.118"`,
   and Cargo must unify that with the version Dioxus needs. If they conflict, file an
   upstream issue to keep the exact pin in upstream's own build (the CLI match matters
   there) while letting library consumers resolve a compatible version. Record the
   outcome in DESIGN.md. Confirm the MSRV.
3. **Gate the model crates' glue upstream.** File, and offer a pull request for, a
   default-on `bindgen` feature on `fc-core`, `fc-model`, `fc-rich-text`, `fc-select`,
   `fc-context-menu`, `fc-grid`, `fc-diff`, and `fc-git`, moving `#[wasm_bindgen]` items
   behind it, with upstream's `build:wasm` keeping it on and its size report unchanged.
4. **Pin and sync.** `upstream.toml` with the revision; `cargo xtask sync-upstream` builds
   upstream's CSS at that revision and copies the CSS into `crates/fc-dioxus/assets/` and
   `tokens.json` and `fixtures.html` into `upstream/`; a test fails when the copies are
   stale.
5. **Lints.** `#![forbid(unsafe_code)]` and `[lints] workspace = true` in every crate, with
   the lint table from AGENTS.md.
6. **Demo skeleton on every renderer.** `fc-demo` renders an empty page through
   `FutureCss`. `cargo xtask demo --renderer <name>` builds it for web, desktop, mobile,
   fullstack, liveview, and native.
7. **The gate.** `cargo xtask check` runs sync verification, clippy, tests, and the demo
   builds; CI runs it on every pull request.

**Gate:** `cargo build --workspace` for the host and `wasm32-unknown-unknown`; the demo
builds for every renderer (mobile may build for one target only in CI); `cargo xtask check`
green. The upstream `bindgen` change is merged and pinned.

## Phase 1 -- Core and demo shell

All of [DESIGN.md, The shared building blocks](DESIGN.md#the-shared-building-blocks):

- `Bem`, `ClassList`, `Modifier`, `fc_modifier!`
- `FcData`, `Lifecycle`, `MessageRole`, `Tone`, `Orientation`, `HeadingLevel`,
  `Availability` with `guard`
- `AccessibleName`, `Alt`, `Clipped`, `Id`, `use_id`
- `use_controllable`
- `DomBridge` for every renderer family (web-sys; eval for desktop, mobile, and liveview;
  `Unsupported` for native and SSR) and the capability context `FutureCss` provides after
  mount
- `FutureCss`, `Theme`, `Brand`, `Direction`, `use_theme`, stylesheet assets
- `testing::render_to_html`, the markup-parity harness, and the coverage test, with every
  component on the pending list

And the demo shell, per [DESIGN.md, The demo](DESIGN.md#the-demo):

- The showcase page: navigation by component, live example plus Rust source per example,
  copy button, theme, scheme, and direction switcher.
- The fixture page: every state, `?dir=rtl` and `?theme=dark` applied before first render.
- The example-file mechanism: one function per file, rendered and shown through
  `include_str!`, with the display fence for imports.
- `tests/demo.rs`: every public component has a showcase section and a fixture section.

**Gate:** unit tests for each piece (including `use_id` determinism across two SSR
renders), and `Divider` complete end to end as the first component: in the showcase with
its source, on the fixture page, passing parity, coverage, and demo tests, and building on
every renderer.

## Phase 2 -- Compliance engine

Before any real component, so every component is audited from its first commit.

1. `fc-tokens`: generator from `upstream/tokens.json`, checked-in output, drift test.
2. `fc-a11y` tree, the `html5ever` adapter (pending the decision below), and the rule
   catalogue in [ACCESSIBILITY.md](ACCESSIBILITY.md#rule-catalogue-initial), each rule with
   its catch and false-positive tests.
3. Reporters: human, JSON, SARIF. CLI `check` and `rules`.
4. Run it over upstream's `fixtures.html`. Findings there are bugs in upstream's reference
   markup: they are filed upstream and fixed there, never suppressed in the rules.
5. `fc_dioxus::testing::assert_accessible`, and the audit of the demo's fixture page in
   `cargo xtask check`.

**Gate:** `fc-a11y check` over the demo's fixture page with zero errors, in the gate.
Upstream's `fixtures.html` has zero errors or a filed issue for each.

## Phase 3 -- Static components

Components whose behaviour is native or absent. Each lands with its demo examples,
fixture states, parity test, a11y assertion, and coverage entry, in pull requests of about
five components:

1. Layout, typography, utilities, divider, description list
2. Button, icon button, link button, badge, tag, kbd, avatar
3. Spinner, progress, progress ring, skeleton, alert, empty state
4. Card, list, breadcrumb, masthead, disclosure (native `<details>`)
5. Chat surface, agent status, reasoning, citations, suggestions
6. Tool call, the five lifecycle panels
7. Diff view (`fc-diff`), git surface (`fc-git`), code block without highlighting

**Gate:** the coverage pending list holds only Phase 4 and 5 components, and every Phase 3
component is in the demo.

## Phase 4 -- Interactive components

1. `behavior::roving`, `dismiss`, `live`, `typeahead`, `sort`, `toast_queue`, `splitter`,
   and the pagination window, each unit-tested; parity tests against upstream's TypeScript
   for the ports.
2. The browser suite pointed at the demo's fixture page: overflow at 320 px, forced
   `:focus-visible`, axe-core, in all four direction and scheme combinations, on web and
   on liveview.
3. Components, in pull requests grouped by shared behaviour, each with its demo examples:
   1. Field, input, textarea, input group, error summary, checkbox, radio, switch, slider
   2. Segmented, button group, control group, toolbar
   3. Tabs, nav, navbar, pagination, carousel
   4. Dialog, sheet, bottom sheet, popover, tooltip, each with its fallback where the
      renderer lacks the top layer
   5. Menu, context menu, menu button, split button, FAB and FAB menu
   6. Select (`fc-select`), search
   7. Toast, table
   8. Context strip, composer
   9. Splitter and resizable (`Split`, `SplitPane`, `Splitter`, `Resizable`)

**Gate:** every Phase 4 component has keyboard scripts generated from its contract table,
and the browser suite passes in all four combinations on web and liveview.

## Phase 5 -- Heavy components

Each needs an upstream crate or a renderer decision, so each is its own pull request. The
upstream issues are filed when the phase starts, and each component starts once its crate
is in the pinned revision:

1. `fc-calendar` upstream, then `DatePicker` and `TimePicker`.
2. `fc-highlight` upstream, then highlighting in `CodeBlock` and in the demo's own
   snippets. Done: `CodeSource::highlighted` uses the pinned crate.
3. `fc-ansi` upstream, then `Terminal`.
4. `fc-markdown` upstream, then Markdown in `StreamingText` and `Message`.
5. `DataGrid` over `fc-grid`, with virtualization from `Viewport`.
6. `Webview`, with the URL safety port.
7. `Editor` over `fc-rich-text`. Last, because `contenteditable` and a virtual DOM disagree
   about who owns the markup. A spike decides how far DOM-patching through `DomBridge`
   reaches across renderers; where it cannot, the editor falls back to a plain `<textarea>`
   over the same document model. The result is recorded in DESIGN.md and `RENDERERS.md`
   before the component is built.

**Gate:** the coverage pending list is empty, and *Waiting on upstream* holds nothing a
listed component needs.

## Phase 6 -- Compliance tooling and renderers

1. The remaining debug-build checks and `AuditMode`.
2. `AuditOverlay` (feature `dev-audit`), itself audited and in the demo.
3. The review ledger: `fc-a11y checklist`, `review`, and `report`.
4. **Renderer support matrix.** Desktop smoke tests under a virtual display, a native
   render smoke test, and a manual mobile pass, producing `docs/RENDERERS.md`: every
   component by every renderer, with the fallback each tier 3 gap takes. Gaps that belong
   to Dioxus are reported there.
5. An example application (a chat client using the composer, context strip, chat surface,
   tool calls, and toasts) audited with every layer: types, debug checks, `fc-a11y check`
   on its SSR output, the browser suite, and a ledger with a manual pass, producing a
   conformance report. It runs on web and desktop at least.

**Gate:** the example's conformance report is generated in CI from real results, with every
criterion either backed by a check or marked as not yet reviewed; `RENDERERS.md` is
generated, not written by hand, except for the manual mobile column.

## Phase 7 -- Release

Coordinated with upstream's release phase, which decides the CSS package names and
registry:

- crates.io publishing for `fc-tokens`, `fc-a11y`, and `fc-dioxus`. Published 0.1.1 on
  2026-10-15, after upstream published its eleven model crates from the revision this
  repository pins ([future-css#58](https://github.com/rexlunae/future-css/pull/58)); the
  order was `fc-tokens`, `fc-a11y`, `fc-dioxus`. 0.1.1 replaces 0.1.0, whose
  `ControlLabel` captured a raw identifier in a format string — a construct current
  nightly rustc rejects, so 0.1.0 did not build on newer toolchains. 0.1.2 (2026-10-16)
  ships the Select popup flip and repins future-css to a6d2846, whose model crates are
  unchanged from their published 0.1.0, so upstream did not republish.
  The 0.1.3 release brings the feature-surface contrast fix at af9f44b, inline
  generated stylesheets for resource-free renderers, native-compatible hidden/open
  attributes, and browser audit timeout diagnostics. Version 0.1.2 was already
  published before its release branch merged, so the combined changes use a new
  patch version.
- docs.rs builds with every feature (configured in the library manifest).
- The demo's web build published per release, linked from the docs and the README.
- A compatibility table between `fc-dioxus` versions, Dioxus versions, and future-css
  versions.

---

## Decisions needed

| Decision | Default this plan assumes | Alternative |
| --- | --- | --- |
| Repository and crate names | Repository `fc-dioxus`; crates `fc-dioxus`, `fc-a11y`, `fc-tokens` | `future-css-dioxus` etc. for crates.io discoverability; decide with upstream's release phase |
| How upstream crates are consumed before they are published | Git dependencies at the revision in `upstream.toml` | A git submodule with path dependencies, which also gives offline builds |
| HTML parser for `fc-a11y` | `html5ever`, behind a default feature | A hand-written parser for well-formed SSR output only, which cannot audit real pages |
| Dioxus version | 0.7, pinned to the minor | Track latest; costs a migration per minor |
| axe-core in the browser suite | Whatever upstream's pending decision lands on | Rely on `fc-a11y` plus keyboard scripts only |
| Where the demo is hosted | GitHub Pages on this repository, one build per release | Alongside upstream's demo |

Settled: the library is a separate repository with future-css as upstream; every Dioxus
renderer is supported, tiered as in [DESIGN.md, Renderers](DESIGN.md#renderers); novel
components are built upstream first; a component is complete only when it is in the demo.

## Risks

- **Upstream turnaround.** Components wait on upstream issues by design. Mitigations: file
  early with a concrete API, offer upstream pull requests where useful, and order each
  phase so blocked work comes last.
- **`wasm-bindgen` unification.** Upstream pins `=0.2.118`. If Dioxus needs a different
  version, the two cannot share one dependency graph; Phase 0 step 2 settles it with
  upstream.
- **Hydration ids.** `use_id` relies on server and client rendering the same tree in the same
  order. Rendering that differs between them (capability checks during render) breaks it;
  capabilities are read only after mount, and the debug-build duplicate-id and
  dangling-idref checks catch the symptom.
- **Renderer gaps.** Webview engines on desktop and mobile vary by OS version, and native
  lacks the top layer, popovers, and `eval`. Fallbacks keep components working; the
  support matrix keeps the gaps visible; mobile stays a manual check until there is a
  harness.
- **Demo upkeep.** Requiring a demo entry for every component adds work to each one. The
  single-file example mechanism keeps it to writing the example once, which the tests
  needed anyway.
- **Scope.** Fifty-five CSS components with every option, on six renderers, is a large
  surface. The coverage, demo, and renderer tests keep "every option" and "every
  renderer" honest; the phase gates keep it finishable.

Select popup placement follows future-css #62: flipped lists use the distance
from the viewport bottom to the trigger top. `tests/platform.test.mjs` runs the
actual host bridge against the pinned upstream binding and checks viewport bounds.

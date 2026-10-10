# AGENTS.md (fc-dioxus)

Operating instructions for contributors to the Dioxus component library, its
accessibility tooling, and its demo: `crates/fc-dioxus`, `crates/fc-a11y`,
`crates/fc-tokens`, and `demo/`.

This repository builds on **future-css** (`rexlunae/future-css`), called *upstream*
throughout. **Read upstream's
[AGENTS.md](https://github.com/rexlunae/future-css/blob/main/AGENTS.md) first.** Its rules
about CSS, tokens, the `data-fc-*` DOM contract, and accessibility still apply to
everything this library renders. This file adds the rules for the Rust component layer.
Where the two overlap, upstream wins on CSS, tokens, and the DOM contract, and this file
wins on Rust.

---

## What this is

A typed Dioxus API over future-css, in the spirit of `dioxus-bulma`: each CSS component is
a Rust component, each modifier an enum variant, and each binding behaviour Rust event
handling over the pure model crates the WASM layer already uses. The design is in
[docs/DESIGN.md](docs/DESIGN.md), the per-component API in
[docs/COMPONENTS.md](docs/COMPONENTS.md), the accessibility enforcement and compliance tools
in [docs/ACCESSIBILITY.md](docs/ACCESSIBILITY.md), and the order of work in
[docs/PLAN.md](docs/PLAN.md).

The CSS is the source of truth. This layer emits the markup the CSS expects and nothing
else. Upstream is pinned to one revision in `upstream.toml`; the model crates, the CSS
copy, and the reference fixtures all come from it.

---

## Non-negotiable invariants

Each is enforced by a test, a lint, or the compiler. If a change needs to break one, stop
and raise it.

1. **No `unsafe`.** `#![forbid(unsafe_code)]` at the top of every crate, and
   `unsafe_code = "forbid"` in the workspace lints. `forbid` cannot be overridden by an inner
   `allow`, which is the point. This includes FFI casts: use checked `dyn_into`, never
   `unchecked_into` or `transmute`.
2. **No panics on consumer input.** No `unwrap`, `expect`, `panic!`, `unreachable!`, or
   slice indexing in library code; clippy's `unwrap_used`, `expect_used`, `panic`, and
   `indexing_slicing` are `deny` outside tests. Bad input renders the documented empty or
   error state and reports in debug builds.
3. **Never emit a class the CSS does not define.** The coverage test parses the CSS and
   fails on an unknown `fc-` class as well as on a missing one.
4. **Upstream first.** A novel component, or anything an existing one needs that upstream
   lacks (a class, modifier, state value, keyboard behaviour, or piece of pure logic), is
   filed as an issue in `rexlunae/future-css` and built there. It is not implemented here,
   not even as a stopgap: it goes under *Waiting on upstream* in `docs/COMPONENTS.md` with
   the issue link, and is wrapped here once upstream ships it. Bugs found in upstream
   components are filed upstream too. See
   [DESIGN.md, Upstream first](docs/DESIGN.md#upstream-first).
5. **No component is complete until it is in the demo:** a showcase section with live
   examples and their Rust source, and a fixture section with every state. `tests/demo.rs`
   fails for a public component without both.
6. **Every Dioxus renderer is supported, at least best effort.** No component calls a
   renderer-specific API; host access goes through `core::platform`, and every platform
   call has a documented fallback. See [DESIGN.md, Renderers](docs/DESIGN.md#renderers).
7. **Never author CSS here.** No stylesheet, `style` rule, or colour is written in Rust or
   in this repository. The one exception is the generated copy of upstream's build output
   in `assets/` (rule 8), which the crate must carry to publish and which is never
   edited. The only inline styles allowed are the custom properties the CSS reads from
   markup (`--fc-slider-pct`, `--fc-progress-value`, `--fc-split-size`,
   `--fc-split-block-size`, `--fc-tool-progress`), set through a typed helper, plus the
   data grid's `grid-template-columns` (`core::measure::grid_columns`), which upstream's
   own binding and fixture write inline because nothing in the CSS creates the tracks.
8. **`crates/fc-dioxus/assets/` and `upstream/` are generated.** Never edit them by hand;
   run `cargo xtask sync-upstream`. A test fails when they differ from what the pinned
   revision produces. Changing what is in them is an upstream change plus a pin bump.
9. **One declaration per modifier.** Every modifier enum goes through `fc_modifier!`.
   Every `data-fc-*` attribute goes through `FcData` or a typed component-specific helper.
   No component spells an `fc-` class or a `data-fc-` name in a string literal outside its
   `Bem` constant.
10. **Logic lives in the upstream model crates, not in components.** Selection,
    typeahead, filtering, ranges, diffs, git status, and text offsets come from
    `fc-select`, `fc-context-menu`, `fc-grid`, `fc-diff`, `fc-git`, `fc-model`, and
    `fc-rich-text`. A component that reimplements one of them is a review failure; a
    function they lack is an upstream issue. Behaviour upstream deliberately keeps in
    TypeScript because only the browser can answer it (the editor's selection work) is
    shipped as upstream wrote it: `cargo xtask sync-upstream` strips the types from the
    files in its `bindings` list into `assets/js/`, and `core::platform` binds them.
11. **A Rust port of TypeScript logic ships with a parity test** against upstream's
    TypeScript at the pinned revision, in the style of upstream's `tests/wasm.test.ts`.
    Two implementations without one will drift, and upstream calls a divergence between
    paths the worst bug this system can have.
12. **Accessibility requirements are types where they can be.** A control whose name cannot
    come from visible children takes a required `AccessibleName`. Images take `Alt`, which
    has no default. A new component that makes an inaccessible use compile is not done. See
    [docs/ACCESSIBILITY.md](docs/ACCESSIBILITY.md#layer-1-types).
13. **Every component example passes `fc-a11y` at AA with zero errors**, and its markup
    matches its `data-fixture` section in upstream's `fixtures.html` structurally.
14. **Never enable the `bare` feature** of a model crate. It is for upstream's
    `build:wasm` alone; the two panic handlers collide in any build that includes std.
15. **Never put a secret, token, or free-form user value in an attribute.** `FcData` has no
    free-form field for this reason. Do not add one.

---

## Rust conventions

- **Edition and MSRV** follow the workspace `Cargo.toml`: the higher of upstream's MSRV and
  the pinned Dioxus's. Dioxus is pinned to one minor version; upgrading it is its own
  pull request.
- **Workspace lints** (added in Phase 0, inherited with `[lints] workspace = true`):

  ```toml
  [workspace.lints.rust]
  unsafe_code = "forbid"
  missing_docs = "deny"
  unreachable_pub = "warn"

  [workspace.lints.clippy]
  pedantic = { level = "warn", priority = -1 }
  unwrap_used = "deny"
  expect_used = "deny"
  panic = "deny"
  indexing_slicing = "deny"
  todo = "deny"
  dbg_macro = "deny"
  ```

  They apply to every crate in this repository from its first commit.
- **Public types derive what they can**: `Debug`, `Clone`, `PartialEq`, and `Eq`, `Hash`,
  `Copy`, `Default` where they make sense. Enums that mirror a CSS vocabulary are
  `#[non_exhaustive]` so the CSS library can add a variant without a breaking release.
- **Newtypes over strings** for anything with rules: `AccessibleName`, `Id`, `Percent`,
  `Timestamp`, `Date`. Validation happens once, in the constructor.
- **Errors** are enums implementing `std::error::Error`. No `Box<dyn Error>` in public APIs,
  no `anyhow` in library crates.
- **Dependencies** are a cost. A new one is justified in the pull request description by
  what it replaces and what it adds to compile time and binary size. Dioxus, `dioxus-ssr`
  (dev), `web-sys` (feature `web`), the upstream model crates, and this workspace's own
  crates are the expected set.
  `fc-a11y` and `fc-tokens` depend on no part of Dioxus.
- **Comments explain why**, as upstream requires. Every public item has a doc comment;
  doc examples compile (`cargo test --doc`).

## Dioxus conventions and gotchas

- **Props are `Clone + PartialEq`**, as memoization requires. Pass large data as
  `ReadSignal<T>` or `Rc<T>`, never a `Vec` that is cloned and compared on every render.
- **Hooks run unconditionally and in order.** No hook inside an `if`, a loop, or an early
  return. `use_id` in particular must be called the same number of times on server and
  client, or hydration ids disagree.
- **Never touch the DOM during render.** Focus, scroll, `showModal`, and popovers happen in
  event handlers or `onmounted`, through `MountedData` or `DomBridge`.
- **`document::eval` never interpolates data into script text.** Send data through the
  eval channel. Script built with `format!` is an injection waiting for a component that
  passes user text.
- **Spread consumer attributes first**, so the component's own `role` and `aria-*` win. A
  consumer adds attributes; it does not silently remove the component's semantics.
- **Events carry domain values** (`EventHandler<String>` for a select), not raw DOM events,
  except on thin native wrappers like `Button`.
- **No component reads `window`, `document`, or `web_sys` directly.** Everything platform
  specific goes through `core::platform`, so every renderer keeps working: web, desktop,
  mobile, SSR and fullstack, liveview, and native.
- **Render the same tree before mount on every renderer.** Capability checks happen after
  mount, through the context `FutureCss` provides, so SSR output and hydration agree.
- **Never use a web-only type in a public signature.** Files, clipboard contents, and other
  host objects come through `core::platform` types with a variant per host family.

---

## Commands

These commands are implemented by the initial workspace bootstrap. See
[docs/PLAN.md](docs/PLAN.md) for the remaining phase gates.

```bash
cargo xtask sync-upstream                     # fetch the pinned future-css, build its CSS,
                                              # copy assets, tokens.json, fixtures.html
cargo test --workspace                        # unit, parity, coverage, demo, a11y
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p fc-a11y -- check upstream/fixtures.html
dx serve -p fc-demo                           # the demo, web renderer
cargo xtask demo --renderer desktop           # the demo on any renderer
cargo xtask browser                           # browser suite against the fixture page
cargo xtask check                             # the gate; runs all of the above
```

`cargo xtask check` is the single gate. Never report a task complete with it failing, and
never weaken a test, rule, or lint to make it pass.

The environment needs the `wasm32-unknown-unknown` target, the `dioxus-cli` matching the
pinned Dioxus version, Node (to build upstream's CSS and run its TypeScript for parity
tests), and Chrome or Chromium for the browser suite. A browser suite that reports no
browser found has skipped its tests; check for that line before calling a run green.

---

## How to add things

### A component

0. **Check that upstream has it.** If future-css has no CSS for it, or it needs anything
   upstream lacks, stop: file an upstream issue, add it to *Waiting on upstream* in
   [docs/COMPONENTS.md](docs/COMPONENTS.md), and build nothing here yet.
1. Read the CSS file, its `data-fixture` section, and its section in upstream's
   `docs/COMPONENTS.md`. The Rust API covers every class, modifier, element, and state
   attribute there.
2. Write or update its entry in [docs/COMPONENTS.md](docs/COMPONENTS.md) first. The API is
   reviewed as a document before it is code.
3. One module in `src/components/`, named after the CSS file. Declare its `Bem` constant
   and its modifier enums with `fc_modifier!`.
4. Build from the shared pieces: `Availability`, `FcData`, `use_id`, `use_controllable`,
   `Clipped`, `behavior::*`. If a piece is missing, add it to `core` or `behavior` for
   everyone rather than inline for one component.
5. Render the native element the fixture uses, the stylesheet asset, and every state the CSS
   styles.
6. **Add it to the demo.** Showcase examples under `demo/src/examples/<component>/`, one
   file per example, each a component function the demo both renders and shows as source;
   and a fixture entry covering every state. The parity test and a11y assertion run off
   the same files. Remove the component from the coverage test's pending list.
7. Add keyboard scripts to the browser suite for anything interactive.
8. Build the demo on every renderer (`cargo xtask demo --renderer all --build-only`) and
   record any tier 3 fallback in `docs/RENDERERS.md`.
9. `cargo xtask check`.

### An accessibility rule

Rules live in `crates/fc-a11y/src/rules/`. Every rule:

- cites the WCAG success criteria it checks, or none if it is a project rule;
- picks its severity honestly: `Error` only for a definite failure;
- has a test with a violation it must catch **and** a legitimate pattern it must not flag;
- has a `help` text that says how to fix it, naming the `fc-dioxus` component that does it
  correctly where there is one.

A rule without a false-positive test will be switched off by the first team it annoys.

### A Rust port of TypeScript logic

Substantial logic belongs upstream: file the issue, and port it into the upstream model
crate where it belongs, or a new upstream crate following upstream's Phase A pattern, with
its parity test there. Only small, Dioxus-shaped behaviour (a sort comparator, a toast
queue) is ported into `fc-dioxus::behavior`, with a parity test here against upstream's
TypeScript at the pinned revision.

### An upstream bump

Change the revision in `upstream.toml`, run `cargo xtask sync-upstream`, and fix what the
parity, coverage, and demo tests report in the same pull request. New upstream components
move from *Waiting on upstream* to their real entry as they are built.

---

## Review checklist

- [ ] Nothing here that upstream should own: no new class, no local CSS, no private copy of
      logic. Gaps are linked upstream issues.
- [ ] No `unsafe`, no panicking calls, clippy clean.
- [ ] Every class and state the CSS defines for this component is reachable from props.
- [ ] No class, colour, or style the CSS does not define.
- [ ] Required names are required in the type, not documented as required.
- [ ] Markup parity, a11y, and coverage tests pass; keyboard scripts exist for interactive
      components.
- [ ] Logic comes from a model crate or a shared `behavior` module.
- [ ] In the demo: showcase examples with their Rust source, and every state on the fixture
      page.
- [ ] Builds on every renderer; host access only through `core::platform`, with fallbacks;
      tier 3 gaps recorded in `docs/RENDERERS.md`.
- [ ] Docs in `docs/COMPONENTS.md` match the code in the same commit.

## Current state

The initial workspace is implemented: pinned CSS/token generation, core types,
`FutureCss`, `Divider`, the demo, four static accessibility rules, and validation
commands. The larger API catalogue in `docs/` remains a target. See the status in
[docs/PLAN.md](docs/PLAN.md) before starting another component. Phase 0's model-crate
feature gate landed in upstream #44 and is pinned. All models are consumed with
default features disabled; never copy that logic here.

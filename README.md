# fc-dioxus

A Dioxus component library built on
[future-css](https://github.com/rexlunae/future-css), in the spirit of `dioxus-bulma`, with
accessibility enforced by its types and checked by its own tooling.

- **Upstream first.** future-css owns the CSS, the reference markup, and the pure models.
  This library wraps what upstream ships; new components are proposed and built there.
- **Every Dioxus renderer:** web, desktop, mobile, SSR and fullstack, liveview, and native,
  in tiers from fully tested to best effort.
- **A demo that shows the Rust.** Every component appears in `fc-demo`, live and with the
  Rust source that renders it. A component is not complete until it is there.

**Status: pre-1.0, published on crates.io (0.1.2).** Every upstream CSS component
has a typed Dioxus component, shown live in the demo with its source, except the theme
picker, which `coverage-pending.toml` tracks. Component APIs may still change between
minor versions.

Add it to an application with `cargo add fc-dioxus`. Each crate has its own README:
[fc-dioxus](crates/fc-dioxus/README.md), [fc-tokens](crates/fc-tokens/README.md) and
[fc-a11y](crates/fc-a11y/README.md).

Start the demo:

```bash
npm ci
dx serve --package fc-demo
```

Open `?fixtures=true&theme=dark&dir=rtl` for the fixture page. Each showcase snippet
is the exact Rust file compiled into the demo. No CSS is authored in this repository.

```bash
cargo xtask check                              # complete local gate
cargo xtask sync-upstream --check              # reproduce the pinned generated files
cargo xtask demo --renderer desktop            # run another renderer
cargo xtask demo --renderer all --build-only   # build all seven configurations
cargo run -p fc-a11y -- check upstream/fixtures.html
cargo run -p fc-demo --no-default-features --features ssr -- 'fixtures=true&theme=dark'
```

The pinned development toolchain installs through rustup. The full gate also needs
Node 22.6+, Dioxus CLI **0.7.9**, Chrome (`CHROME_PATH` can override its path),
and Xcode with the `aarch64-apple-ios-sim` Rust target for the mobile build.
The committed CI workflow runs that gate on macOS. Missing tools fail explicitly;
the browser suite does not silently skip.

The upstream pin currently includes the fixture ID fix in
[future-css#46](https://github.com/rexlunae/future-css/pull/46) and the merged
[bindgen feature gate](https://github.com/rexlunae/future-css/pull/44).
fc-dioxus pins nine upstream model crates with their JavaScript glue disabled.
Eight are re-exported through `fc_dioxus::models`; `fc-highlight` stays internal
to `CodeBlock`.
See [the renderer build record](docs/RENDERERS.md) for the distinction between
successful compilation and runtime verification.

| Document | What it covers |
| --- | --- |
| [AGENTS.md](AGENTS.md) | Rules for anyone, human or agent, changing these crates |
| [docs/DESIGN.md](docs/DESIGN.md) | Architecture, crates, shared building blocks, conventions, testing |
| [docs/COMPONENTS.md](docs/COMPONENTS.md) | The Rust API for every CSS component and every option |
| [docs/ACCESSIBILITY.md](docs/ACCESSIBILITY.md) | How WCAG 2.2 AA is enforced, and the `fc-a11y` compliance tools |
| [docs/PLAN.md](docs/PLAN.md) | Phases, exit gates, open decisions, risks |

A minimal example using the implemented API:

```rust
use fc_dioxus::prelude::*;

#[component]
fn App() -> Element {
    rsx! {
        FutureCss { theme: Theme::System,
            p { "First section" }
            Divider { inset: DividerInset::Middle }
            p { "Next section" }
        }
    }
}
```

`fc-a11y` currently implements four static rules; it does not claim complete WCAG
coverage. The browser fixture suite additionally runs axe-core's WCAG A/AA checks.
Library crates do not select a renderer by default. Dependency choices and the
verified Rust 1.88 minimum are recorded in [DEPENDENCIES.md](docs/DEPENDENCIES.md).

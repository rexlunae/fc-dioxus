# Dependencies in the initial workspace

`fc-dioxus` uses Dioxus 0.7.9, `fc-tokens`, optional `web-sys` for entrypoint
query inputs, and optional SSR/audit dependencies behind `testing`.
Renderer selection belongs to the application, so the library's default features
do not pull in a browser, webview, server, or native renderer.

`fc-a11y` has no Dioxus dependency. Its default `html` feature uses `html5ever`
0.35 and `markup5ever_rcdom` 0.35. These replace a handwritten parser and preserve
browser error recovery, entity decoding, and optional-end-tag behavior. They add
parser code and compilation only to the CLI and tests; they do not enter the
normal demo/application binary. `--no-default-features` keeps the rule engine
over its own tree without the parser. `fc-tokens` has no dependencies.

The workspace command runner uses only Rust's standard library; its Node scripts
use built-in modules. `axe-core` 4.14.0 is an exact, locked development dependency
for headless browser audits and is never shipped with the app.

All upstream CSS, generated token values, and eight model crate dependencies come
from `upstream.toml`. Sync rewrites the marked model block in the library manifest.
The models replace future local copies of selection, diff, grid, and text logic.
Their JavaScript glue and standalone WASM runtime are disabled; their pure Rust
code adds compilation work, while unused operations are removed by the linker.
`cargo xtask models` checks those dependency graphs and actual native/wasm32
library builds. Upstream owns their TypeScript parity tests.

The workspace MSRV is 1.88. The upstream model crates advertise 1.82 and Dioxus
advertises 1.83, but the locked Dioxus dependency graph includes
`const-serialize-macro` 0.8.0-alpha.1, which uses let chains requiring Rust 1.88.
An actual Rust 1.86 check reproduced this failure; Rust 1.88 checks the workspace.
The native renderer separately requires at least 1.86. The development toolchain
is pinned to 1.94.1 so formatter and Clippy output stay reproducible.

Development/test profiles omit debug symbols and incremental caches to keep the
seven renderer builds practical on a small disk. This changes neither checks nor
release optimizations. Crate names and registry publishing remain Phase 7 work.

# fc-dioxus

Typed, accessible [Dioxus](https://dioxuslabs.com) 0.7 components over
[future-css](https://github.com/rexlunae/future-css). The stylesheets, reference markup
and pure Rust models are upstream's; this crate wraps them in components whose props
make the accessible structure the default and an inaccessible one hard to express.

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

`FutureCss` loads the stylesheets through Dioxus assets and sets the theme. The library
selects no renderer: the application enables web, desktop, mobile, fullstack, liveview,
SSR or native in its own `dioxus` dependency.

| Feature | Effect |
| --- | --- |
| `web` | Reads entrypoint query inputs through `web-sys` |
| `router` | Links navigate through the nearest `dioxus-router` `Router` |
| `testing` | SSR rendering helpers with `fc-a11y` structural checks |

The minimum supported Rust version is 1.88. Status is pre-1.0: component APIs may
change between minor versions. The
[repository](https://github.com/rexlunae/fc-dioxus) has the live demo, the full
component reference and the accessibility policy.

Licensed under MIT.

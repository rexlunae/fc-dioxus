# fc-tokens

The design token vocabulary of [future-css](https://github.com/rexlunae/future-css) as a
Rust enum, with each token's custom property name, `var()` reference and light and dark
values. It is generated from upstream's token build and has no dependencies, so it can be
used without a UI framework. [fc-dioxus](https://crates.io/crates/fc-dioxus) re-exports it
as `fc_dioxus::tokens`.

```rust
use fc_tokens::{Scheme, Token};

for token in Token::ALL {
    println!("{} = {}", token.name(), token.value(Scheme::Dark));
}
```

Licensed under MIT.

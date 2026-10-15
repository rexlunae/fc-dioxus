# fc-a11y

Framework-independent static accessibility checks over HTML: image alternatives, ARIA
references and their uniqueness, and positive `tabindex`. It reports which WCAG success
criteria each rule covers and does not claim full WCAG conformance.

```bash
cargo install fc-a11y
fc-a11y rules
fc-a11y check page.html
```

The default `html` feature parses with `html5ever`. With `default-features = false` the
rule engine runs over its own element tree and has no dependencies beyond
[fc-tokens](https://crates.io/crates/fc-tokens).

Licensed under MIT.

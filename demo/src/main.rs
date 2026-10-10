//! The fc-dioxus demo.
#![forbid(unsafe_code)]

#[cfg(feature = "ssr")]
fn main() {
    let query = std::env::args().nth(1).unwrap_or_default();
    println!(
        "{}",
        fc_demo::render_page(fc_demo::DemoOptions::from_query(&query))
    );
}

#[cfg(all(
    not(feature = "ssr"),
    any(
        feature = "web",
        feature = "desktop",
        feature = "mobile",
        feature = "native",
        feature = "server",
        feature = "liveview"
    )
))]
fn main() {
    dioxus::launch(fc_demo::App);
}

#[cfg(not(any(
    feature = "ssr",
    feature = "web",
    feature = "desktop",
    feature = "mobile",
    feature = "native",
    feature = "server",
    feature = "liveview"
)))]
fn main() {
    eprintln!("Select a renderer with cargo xtask demo --renderer <name>.");
}

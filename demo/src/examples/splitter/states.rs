use fc_dioxus::prelude::*;

// example:start
/// Side-by-side and stacked splits, a collapsed pane at the inline end, edge handles in
/// pixels, a long scrollable pane, and a fixed split. Focus a bar and use the arrows,
/// Home, End, or Enter, or drag it.
#[component]
pub fn States() -> Element {
    const STACK: Bem = Bem::new("fc-stack");
    let mut files = use_signal(|| 35.0);
    let mut settled = use_signal(|| 35.0);
    rsx! {
        div { class: STACK.block(),
            Split { label: name!("File list"), min: 15.0, max: 70.0, value: ReadSignal::from(files),
                onresize: move |resize: Resize| { files.set(resize.value); if resize.settled { settled.set(resize.value); } },
                primary: rsx! { "Files" }, secondary: rsx! { "Editor" } }
            p { role: "status", "File list: {settled}% (saved when you let go)" }
            Split { label: name!("Code"), stacked: true, block_size: "10rem", default_value: 60.0, min: 20.0, max: 80.0,
                primary: rsx! { "Code" }, secondary: rsx! { "Terminal" } }
            Split { label: name!("Context"), primary_last: true, default_value: 0.0, min: 0.0, max: 60.0,
                primary: rsx! { "Context" }, secondary: rsx! { "Chat" } }
            Resizable { label: name!("Panel"), default_value: 192.0, min: 120.0, max: 288.0, "Panel" }
            Resizable { label: name!("Notes"), block: true, default_value: 80.0, min: 48.0, max: 240.0, "Notes" }
            Split { label: name!("History"), default_value: 25.0, min: 10.0, max: 90.0, primary_label: name!("History pane"),
                primary: rsx! { "Conversation history with a very long title that wraps, and an unbroken path /workspace/projects/future-css/src/components/splitter.css" },
                secondary: rsx! { "The reply pane keeps its own scroller, so a long answer here scrolls inside the pane instead of pushing the bar or the page sideways." } }
            Split { label: name!("Fixed"), availability: Availability::Disabled, primary: rsx! { "Fixed" }, secondary: rsx! { "Pane" } }
        }
    }
}

use fc_dioxus::prelude::*;

// example:start
/// Addons, and icon buttons inside the field at the start and the end.
#[component]
pub fn States() -> Element {
    const STACK: Bem = Bem::new("fc-stack");
    let mut query = use_signal(|| "release notes".to_owned());
    let mut revealed = use_signal(|| false);
    let reason = use_id();
    rsx! {
        div { class: STACK.block(),
            InputGroup {
                InputAddon { decorative: true, "$" }
                Input { aria_label: "Budget", inputmode: "decimal", default_value: "40" }
                InputAddon { "USD" }
            }
            InputGroup {
                InputAddon { "https://" }
                Input { aria_label: "Domain", invalid: true, default_value: "exa mple.com" }
            }
            // Before the input is the start, after it the end, in either direction.
            InputGroup {
                InputAction { icon: IconName::Search, label: name!("Search") }
                Input { r#type: InputType::Search, aria_label: "Search chats", value: ReadSignal::from(query), oninput: move |text| query.set(text) }
                InputAction { icon: IconName::Close, label: name!("Clear search"), onclick: move |_| query.set(String::new()) }
            }
            // A toggle keeps its name; aria-pressed and the icon say whether it is on.
            InputGroup {
                Input { r#type: if revealed() { InputType::Text } else { InputType::Password }, aria_label: "API key", default_value: "hunter2" }
                InputAction { icon: if revealed() { IconName::EyeOff } else { IconName::Eye }, label: name!("Show API key"),
                    pressed: revealed(), onclick: move |_| revealed.toggle() }
            }
            InputGroup {
                Input { aria_label: "API key", default_value: "hunter2" }
                InputAction { icon: IconName::EyeOff, label: name!("Show API key"), pressed: true }
            }
            InputGroup {
                InputAction { icon: IconName::Attach, label: name!("Attach a file") }
                Input { aria_label: "Message", invalid: true }
                InputAction { icon: IconName::Send, label: name!("Send"), availability: Availability::Unavailable { reason: reason.clone() } }
            }
            p { id: "{reason}", "Write a message to send it." }
            InputGroup {
                InputAddon { "https://" }
                Input { aria_label: "Share link", default_value: "example.com/s/42", availability: Availability::Disabled }
                InputAction { icon: IconName::Copy, label: name!("Copy link"), availability: Availability::Disabled }
            }
        }
    }
}

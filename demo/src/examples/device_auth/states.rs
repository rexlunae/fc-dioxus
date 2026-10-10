use super::super::{long_path, record};
use fc_dioxus::prelude::*;

// example:start
/// A device sign-in in every lifecycle state. The code is text only: it is a credential,
/// so it never goes into an attribute.
#[component]
pub fn States() -> Element {
    const GRID: Bem = Bem::new("fc-grid-auto");
    const LINK: Bem = Bem::new("fc-btn");
    const SECONDARY: Bem = Bem::new("fc-btn--secondary");
    rsx! {
        div { class: GRID.block(),
            for (index, state) in Lifecycle::ALL.iter().copied().enumerate() {
                DeviceAuth { key: "{index}", label: name!("Connect your device"), record: record("device-auth", state, index),
                    status: rsx! { p { "{state:?}" } }, title: rsx! { "Connect your device" }, code: "ABCD-1234".to_owned(),
                    meta: rsx! { "Expires in 5 minutes." },
                    p { "Open the provider’s verification page and enter this example code." }
                    a { class: "{LINK.block()} {SECONDARY.block()}", href: "https://example.com/activate", "Verification page" }
                }
            }
            DeviceAuth { label: name!("Authorize the workspace"), region: true, record: record("device-auth", Lifecycle::Pending, 100),
                title: rsx! { "Authorize the extended multilingual workspace session 日本語 العربية" }, code: "EXAMPLE-ONLY-1234".to_owned(),
                a { href: "https://example.com/activate", "https://example.com/activate/{long_path()}" }
            }
            DeviceAuth { label: name!("No active authorization"), region: true, record: record("device-auth", Lifecycle::Complete, 101),
                title: rsx! { "No active authorization" }, p { "No verification code has been issued." } }
            DeviceAuth { label: name!("Sign in on your TV"), title: rsx! { "Sign in on your TV" }, code: "WXYZ-9876".to_owned(),
                record: Record { state: Some(Lifecycle::Running), ..Record::default() },
                meta: rsx! { "Waiting for you to approve on the other device." },
                actions: rsx! { Button { variant: ButtonVariant::Ghost, "Use a different method" } },
                p { "Enter this code on your TV." }
            }
        }
    }
}

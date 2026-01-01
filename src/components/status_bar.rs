use crate::dns::AddressFamily;
use crate::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn StatusBar(state: Signal<AppState>) -> Element {
    let current_state = state.read().current_dns_state.clone();
    let message = state.read().message.clone();

    rsx! {
        div { class: "status-bar",
            if let Some(msg) = message {
                div {
                    class: if msg.is_error { "message error" } else { "message success" },
                    "{msg.text}"
                }
            }

            div { class: "status-section",
                div { class: "status-label", "Current IPv4 DNS:" }
                div { class: "status-value", "{current_state.get_display(AddressFamily::IPv4)}" }
            }

            div { class: "status-section",
                div { class: "status-label", "Current IPv6 DNS:" }
                div { class: "status-value", "{current_state.get_display(AddressFamily::IPv6)}" }
            }
        }
    }
}

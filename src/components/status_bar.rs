use crate::dns::AddressFamily;
use crate::state::{AppState, MessageLevel};
use dioxus::prelude::*;

#[component]
pub fn StatusBar(state: Signal<AppState>) -> Element {
    let (current_state, message) = {
        let read_state = state.read();
        (
            read_state.current_dns_state.clone(),
            read_state.message.clone(),
        )
    };

    rsx! {
        div { class: "status-bar",
            if let Some(msg) = message {
                {
                    let class_name = match msg.level {
                        MessageLevel::Success => "message success",
                        MessageLevel::Warning => "message warning",
                        MessageLevel::Error => "message error",
                    };
                    rsx! {
                        div {
                            class: "{class_name}",
                            span { class: "message-text", "{msg.text}" }
                            button {
                                r#type: "button",
                                class: "message-close-btn",
                                aria_label: "Close message",
                                title: "Close",
                                onclick: move |_| state.write().clear_message(),
                                "×"
                            }
                        }
                    }
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

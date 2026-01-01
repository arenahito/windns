use crate::dns::DnsMode;
use crate::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn DnsModeSelector(state: Signal<AppState>, on_change: EventHandler<DnsMode>) -> Element {
    let current_mode = state.read().dns_mode;

    rsx! {
        div { class: "section",
            div { class: "section-title", "DNS Mode" }
            div { class: "radio-group",
                div { class: "radio-option",
                    input {
                        r#type: "radio",
                        id: "mode-automatic",
                        name: "dns-mode",
                        checked: current_mode == DnsMode::Automatic,
                        onchange: move |_| on_change.call(DnsMode::Automatic)
                    }
                    label { r#for: "mode-automatic", "Automatic (DHCP)" }
                }
                div { class: "radio-option",
                    input {
                        r#type: "radio",
                        id: "mode-manual",
                        name: "dns-mode",
                        checked: current_mode == DnsMode::Manual,
                        onchange: move |_| on_change.call(DnsMode::Manual)
                    }
                    label { r#for: "mode-manual", "Manual" }
                }
                div { class: "radio-option",
                    input {
                        r#type: "radio",
                        id: "mode-manual-doh",
                        name: "dns-mode",
                        checked: current_mode == DnsMode::ManualDoH,
                        onchange: move |_| on_change.call(DnsMode::ManualDoH)
                    }
                    label { r#for: "mode-manual-doh", "Manual (DoH)" }
                }
            }
        }
    }
}

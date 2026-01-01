use crate::dns::DnsMode;
use dioxus::prelude::*;

#[component]
pub fn DnsModeSelector(current_mode: DnsMode, on_change: EventHandler<DnsMode>) -> Element {
    rsx! {
        div { class: "dns-mode-radio-group",
            div { class: "radio-group horizontal",
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
            }
        }
    }
}

use crate::dns::{AddressFamily, DnsMode};
use crate::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn DnsInput(
    state: Signal<AppState>,
    on_enabled_change: EventHandler<bool>,
    on_primary_change: EventHandler<String>,
    on_secondary_change: EventHandler<String>,
    on_doh_template_change: EventHandler<String>,
) -> Element {
    let dns_mode = state.read().dns_mode;
    let active_tab = state.read().active_tab;
    let entry = state.read().get_current_entry().clone();

    let is_automatic = dns_mode == DnsMode::Automatic;
    let is_doh_mode = dns_mode == DnsMode::ManualDoH;
    let is_disabled = is_automatic || !entry.enabled;

    let family_label = match active_tab {
        AddressFamily::IPv4 => "IPv4",
        AddressFamily::IPv6 => "IPv6",
    };

    let placeholder_primary = match active_tab {
        AddressFamily::IPv4 => "e.g., 8.8.8.8",
        AddressFamily::IPv6 => "e.g., 2001:4860:4860::8888",
    };

    let placeholder_secondary = match active_tab {
        AddressFamily::IPv4 => "e.g., 8.8.4.4",
        AddressFamily::IPv6 => "e.g., 2001:4860:4860::8844",
    };

    rsx! {
        div { class: "section",
            div { class: "section-title", "{family_label} DNS Settings" }

            div { class: "checkbox-group",
                input {
                    r#type: "checkbox",
                    id: "dns-enabled",
                    checked: entry.enabled,
                    disabled: is_automatic,
                    onchange: move |evt| on_enabled_change.call(evt.checked())
                }
                label { r#for: "dns-enabled", "Enable {family_label} DNS" }
            }

            div { class: "dns-inputs",
                div { class: "form-group",
                    label { r#for: "primary-dns", "Primary DNS Server" }
                    input {
                        r#type: "text",
                        id: "primary-dns",
                        placeholder: "{placeholder_primary}",
                        value: "{entry.primary}",
                        disabled: is_disabled,
                        oninput: move |evt| on_primary_change.call(evt.value())
                    }
                }

                div { class: "form-group",
                    label { r#for: "secondary-dns", "Secondary DNS Server (Optional)" }
                    input {
                        r#type: "text",
                        id: "secondary-dns",
                        placeholder: "{placeholder_secondary}",
                        value: "{entry.secondary}",
                        disabled: is_disabled,
                        oninput: move |evt| on_secondary_change.call(evt.value())
                    }
                }

                if is_doh_mode {
                    div { class: "form-group doh-template-group",
                        label { r#for: "doh-template", "DoH Template URL (Optional)" }
                        input {
                            r#type: "text",
                            id: "doh-template",
                            placeholder: "https://dns.google/dns-query{{?dns}}",
                            value: "{entry.doh_template}",
                            disabled: is_disabled,
                            oninput: move |evt| on_doh_template_change.call(evt.value())
                        }
                        div { class: "input-hint",
                            "DoH template must start with https:// and contain {{?dns}}"
                        }
                    }
                }
            }
        }
    }
}

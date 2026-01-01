use crate::components::{DnsModeSelector, ProfileSelector};
use crate::dns::{AddressFamily, DnsMode, DnsSettings, DohMode};
use crate::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn DnsInput(
    state: Signal<AppState>,
    on_settings_change: EventHandler<DnsSettings>,
    on_mode_change: EventHandler<DnsMode>,
    on_profile_change: EventHandler<String>,
    on_new_profile: EventHandler<()>,
    on_profile_name_change: EventHandler<String>,
    on_delete_profile: EventHandler<()>,
) -> Element {
    let dns_mode = state.read().dns_mode;
    let settings = state.read().current_settings.clone();
    let interface = state.read().selected_interface().cloned();

    let is_automatic = dns_mode == DnsMode::Automatic;

    let (has_ipv4, has_ipv6) = interface
        .map(|i| (i.has_ipv4, i.has_ipv6))
        .unwrap_or((false, false));

    rsx! {
        div { class: "section",
            h2 { class: "section-title", "DNS Settings" }
            DnsModeSelector { current_mode: dns_mode, on_change: on_mode_change }

            ProfileSelector {
                state: state,
                disabled: is_automatic,
                on_profile_change: on_profile_change,
                on_new_profile: on_new_profile,
                on_name_change: on_profile_name_change,
                on_delete: on_delete_profile,
            }

            div { class: "dns-settings-grid",
                if has_ipv4 {
                    DnsFamilyPanel {
                        family: AddressFamily::IPv4,
                        entry: settings.ipv4.clone(),
                        disabled: is_automatic,
                        on_change: move |entry| {
                            let mut new_settings = state.read().current_settings.clone();
                            new_settings.ipv4 = entry;
                            on_settings_change.call(new_settings);
                        },
                    }
                }
                if has_ipv6 {
                    DnsFamilyPanel {
                        family: AddressFamily::IPv6,
                        entry: settings.ipv6.clone(),
                        disabled: is_automatic,
                        on_change: move |entry| {
                            let mut new_settings = state.read().current_settings.clone();
                            new_settings.ipv6 = entry;
                            on_settings_change.call(new_settings);
                        },
                    }
                }
            }
        }
    }
}

#[component]
fn DnsFamilyPanel(
    family: AddressFamily,
    entry: crate::dns::DnsEntry,
    disabled: bool,
    on_change: EventHandler<crate::dns::DnsEntry>,
) -> Element {
    let family_label = family.as_str();
    let is_disabled = disabled || !entry.enabled;

    let (placeholder_primary, placeholder_secondary) = match family {
        AddressFamily::IPv4 => ("e.g., 8.8.8.8", "e.g., 8.8.4.4"),
        AddressFamily::IPv6 => ("e.g., 2001:4860:4860::8888", "e.g., 2001:4860:4860::8844"),
    };

    let id_prefix = match family {
        AddressFamily::IPv4 => "ipv4",
        AddressFamily::IPv6 => "ipv6",
    };

    rsx! {
        div { class: "dns-family-panel",
            div { class: "dns-family-header",
                span { class: "dns-family-title", "{family_label}" }
                label { class: "toggle-switch",
                    input {
                        r#type: "checkbox",
                        checked: entry.enabled,
                        disabled: disabled,
                        onchange: {
                            let entry = entry.clone();
                            move |evt: Event<FormData>| {
                                let mut new_entry = entry.clone();
                                new_entry.enabled = evt.checked();
                                on_change.call(new_entry);
                            }
                        },
                    }
                    span { class: "toggle-slider" }
                }
            }

            DnsServerInput {
                id_prefix: format!("{}-primary", id_prefix),
                label: "Primary DNS",
                placeholder: placeholder_primary.to_string(),
                server: entry.primary.clone(),
                disabled: is_disabled,
                on_change: {
                    let entry = entry.clone();
                    move |server| {
                        let mut new_entry = entry.clone();
                        new_entry.primary = server;
                        on_change.call(new_entry);
                    }
                },
            }

            DnsServerInput {
                id_prefix: format!("{}-secondary", id_prefix),
                label: "Secondary DNS",
                placeholder: placeholder_secondary.to_string(),
                server: entry.secondary.clone(),
                disabled: is_disabled,
                on_change: {
                    let entry = entry.clone();
                    move |server| {
                        let mut new_entry = entry.clone();
                        new_entry.secondary = server;
                        on_change.call(new_entry);
                    }
                },
            }
        }
    }
}

#[component]
fn DnsServerInput(
    id_prefix: String,
    label: String,
    placeholder: String,
    server: crate::dns::DnsServerEntry,
    disabled: bool,
    on_change: EventHandler<crate::dns::DnsServerEntry>,
) -> Element {
    let doh_enabled = server.doh_mode == DohMode::On;

    rsx! {
        div { class: "dns-server-section",
            div { class: "form-group",
                label { r#for: "{id_prefix}-address", "{label}" }
                input {
                    r#type: "text",
                    id: "{id_prefix}-address",
                    placeholder: "{placeholder}",
                    value: "{server.address}",
                    disabled: disabled,
                    oninput: {
                        let server = server.clone();
                        move |evt: Event<FormData>| {
                            let mut new_server = server.clone();
                            new_server.address = evt.value();
                            on_change.call(new_server);
                        }
                    },
                }
            }

            div { class: "form-group",
                label { r#for: "{id_prefix}-doh", "DNS over HTTPS" }
                select {
                    id: "{id_prefix}-doh",
                    disabled: disabled,
                    value: if doh_enabled { "on" } else { "off" },
                    onchange: {
                        let server = server.clone();
                        move |evt: Event<FormData>| {
                            let mut new_server = server.clone();
                            new_server.doh_mode = if evt.value() == "on" {
                                DohMode::On
                            } else {
                                DohMode::Off
                            };
                            on_change.call(new_server);
                        }
                    },
                    option { value: "off", "Off" }
                    option { value: "on", "On (manual template)" }
                }
            }

            if doh_enabled {
                div { class: "doh-options",
                    div { class: "form-group",
                        label { r#for: "{id_prefix}-template", "DoH Template URL" }
                        input {
                            r#type: "text",
                            id: "{id_prefix}-template",
                            placeholder: "https://dns.example.com/dns-query",
                            value: "{server.doh_template}",
                            disabled: disabled,
                            oninput: {
                                let server = server.clone();
                                move |evt: Event<FormData>| {
                                    let mut new_server = server.clone();
                                    new_server.doh_template = evt.value();
                                    on_change.call(new_server);
                                }
                            },
                        }
                    }

                    div { class: "checkbox-group",
                        input {
                            r#type: "checkbox",
                            id: "{id_prefix}-fallback",
                            checked: server.allow_fallback,
                            disabled: disabled,
                            onchange: {
                                let server = server.clone();
                                move |evt: Event<FormData>| {
                                    let mut new_server = server.clone();
                                    new_server.allow_fallback = evt.checked();
                                    on_change.call(new_server);
                                }
                            },
                        }
                        label { r#for: "{id_prefix}-fallback", "Allow fallback to plaintext" }
                    }
                }
            }
        }
    }
}

use crate::components::*;
use crate::dns::{
    AddressFamily, DnsMode, DnsSettings, get_current_dns, get_network_interfaces, load_config,
    save_config, set_dns_automatic, set_dns_with_doh,
};
use crate::state::{AppState, Message};
use dioxus::prelude::*;

#[allow(non_snake_case)]
pub fn App() -> Element {
    let mut state = use_signal(AppState::new);

    use_effect(move || {
        spawn(async move {
            initialize_app(state).await;
        });
    });

    let on_interface_change = move |index: usize| {
        spawn(async move {
            change_interface(state, index).await;
        });
    };

    let on_mode_change = move |mode: DnsMode| {
        spawn(async move {
            change_dns_mode(state, mode).await;
        });
    };

    let on_settings_change = move |settings: DnsSettings| {
        state.write().current_settings = settings;
    };

    let on_apply = move |_| {
        spawn(async move {
            apply_dns_settings(state).await;
        });
    };

    let on_reset = move |_| {
        spawn(async move {
            reset_dns_settings(state).await;
        });
    };

    rsx! {
        style { {include_str!("../assets/main.css")} }
        div { class: "app-container",
            div { class: "content",
                NetworkSelector {
                    state: state,
                    on_change: on_interface_change
                }
                DnsInput {
                    state: state,
                    on_settings_change: on_settings_change,
                    on_mode_change: on_mode_change
                }
                ActionButtons {
                    state: state,
                    on_apply: on_apply,
                    on_reset: on_reset
                }
            }
            StatusBar { state: state }
        }
    }
}

async fn initialize_app(mut state: Signal<AppState>) {
    state.write().clear_message();

    match load_config() {
        Ok(config) => {
            state.write().config = config;
        }
        Err(e) => {
            state
                .write()
                .set_message(Message::error(format!("Failed to load config: {}", e)));
        }
    }

    match get_network_interfaces() {
        Ok(interfaces) => {
            if interfaces.is_empty() {
                state
                    .write()
                    .set_message(Message::error("No network interfaces found"));
                return;
            }
            state.write().interfaces = interfaces;
            state.write().selected_interface_index = 0;

            refresh_current_dns(state).await;
        }
        Err(e) => {
            state.write().set_message(Message::error(format!(
                "Failed to get network interfaces: {}",
                e
            )));
        }
    }
}

async fn change_interface(mut state: Signal<AppState>, index: usize) {
    {
        let mut write_state = state.write();
        write_state.selected_interface_index = index;
        write_state.clear_message();
        write_state.dns_mode = DnsMode::Automatic;
        write_state.load_settings_for_mode(DnsMode::Automatic);
    }

    refresh_current_dns(state).await;
}

async fn change_dns_mode(mut state: Signal<AppState>, mode: DnsMode) {
    let old_mode = state.read().dns_mode;

    if old_mode == mode {
        return;
    }

    if old_mode == DnsMode::Manual {
        let config = {
            let mut write_state = state.write();
            write_state.save_settings_for_mode(old_mode);
            write_state.config.clone()
        };

        if let Err(e) = save_config(&config) {
            state
                .write()
                .set_message(Message::error(format!("Failed to save config: {}", e)));
            return;
        }
    }

    {
        let mut write_state = state.write();
        write_state.dns_mode = mode;
        write_state.load_settings_for_mode(mode);
        write_state.clear_message();
    }
}

async fn refresh_current_dns(mut state: Signal<AppState>) {
    let interface_index = {
        let read_state = state.read();
        read_state.selected_interface().map(|i| i.interface_index)
    };

    if let Some(index) = interface_index {
        match get_current_dns(index).await {
            Ok(dns_state) => {
                state.write().current_dns_state = dns_state;
            }
            Err(e) => {
                state
                    .write()
                    .set_message(Message::error(format!("Failed to get current DNS: {}", e)));
            }
        }
    }
}

async fn apply_dns_settings(mut state: Signal<AppState>) {
    let validation_result = {
        let mut write_state = state.write();
        write_state.clear_message();
        write_state.validate_current_settings()
    };

    if let Err(e) = validation_result {
        state.write().set_message(Message::error(e));
        return;
    }

    state.write().set_loading(true);

    let result = apply_dns_settings_impl(&state).await;

    state.write().set_loading(false);

    match result {
        Ok(()) => {
            let config = {
                let mut write_state = state.write();
                write_state.set_message(Message::success("DNS settings applied successfully"));
                let dns_mode = write_state.dns_mode;
                write_state.save_settings_for_mode(dns_mode);
                write_state.config.clone()
            };

            if let Err(e) = save_config(&config) {
                state.write().set_message(Message::error(format!(
                    "Settings applied but failed to save config: {}",
                    e
                )));
            }

            refresh_current_dns(state).await;
        }
        Err(e) => {
            state.write().set_message(Message::error(format!(
                "Failed to apply DNS settings: {}",
                e
            )));
        }
    }
}

async fn apply_dns_settings_impl(state: &Signal<AppState>) -> Result<(), String> {
    let interface = state
        .read()
        .selected_interface()
        .ok_or("No interface selected")?
        .clone();

    let interface_index = interface.interface_index;
    let interface_guid = &interface.interface_guid;
    let dns_mode = state.read().dns_mode;
    let settings = state.read().current_settings.clone();

    match dns_mode {
        DnsMode::Automatic => {
            if interface.has_ipv4 {
                set_dns_automatic(interface_index, AddressFamily::IPv4)
                    .await
                    .map_err(|e| e.to_string())?;
            }
            if interface.has_ipv6 {
                set_dns_automatic(interface_index, AddressFamily::IPv6)
                    .await
                    .map_err(|e| e.to_string())?;
            }
        }
        DnsMode::Manual => {
            if interface.has_ipv4 && settings.ipv4.enabled {
                set_dns_with_doh(
                    interface_index,
                    interface_guid,
                    AddressFamily::IPv4,
                    &settings.ipv4,
                )
                .await
                .map_err(|e| e.to_string())?;
            } else if interface.has_ipv4 {
                set_dns_automatic(interface_index, AddressFamily::IPv4)
                    .await
                    .map_err(|e| e.to_string())?;
            }

            if interface.has_ipv6 && settings.ipv6.enabled {
                set_dns_with_doh(
                    interface_index,
                    interface_guid,
                    AddressFamily::IPv6,
                    &settings.ipv6,
                )
                .await
                .map_err(|e| e.to_string())?;
            } else if interface.has_ipv6 {
                set_dns_automatic(interface_index, AddressFamily::IPv6)
                    .await
                    .map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(())
}

async fn reset_dns_settings(mut state: Signal<AppState>) {
    let mut write_state = state.write();
    write_state.clear_message();

    let mode = write_state.dns_mode;
    write_state.load_settings_for_mode(mode);
    write_state.set_message(Message::success("Settings reset to saved values"));
}

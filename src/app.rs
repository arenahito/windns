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
        change_dns_mode(state, mode);
    };

    let on_settings_change = move |settings: DnsSettings| {
        state.write().current_settings = settings;
    };

    let on_profile_change = move |id: String| {
        state.write().select_profile(&id);
    };

    let on_new_profile = move |_| {
        state.write().create_new_profile();
    };

    let on_profile_name_change = move |name: String| {
        state.write().current_profile_name = name;
    };

    let on_delete_profile = move |_| {
        state.write().show_delete_confirm = true;
    };

    let on_confirm_delete = move |_| {
        let mut write_state = state.write();
        write_state.delete_current_profile();
        write_state.show_delete_confirm = false;
    };

    let on_cancel_delete = move |_| {
        state.write().show_delete_confirm = false;
    };

    let on_save = move |_| {
        spawn(async move {
            save_settings_only(state).await;
        });
    };

    let on_apply = move |_| {
        spawn(async move {
            apply_dns_settings(state).await;
        });
    };

    let show_delete_confirm = state.read().show_delete_confirm;
    let profile_name_for_dialog = state.read().current_profile_name.clone();

    rsx! {
        style { {include_str!("../assets/main.css")} }

        if show_delete_confirm {
            DeleteConfirmDialog {
                profile_name: profile_name_for_dialog,
                on_confirm: on_confirm_delete,
                on_cancel: on_cancel_delete,
            }
        }

        div { class: "app-container",
            div { class: "content",
                NetworkSelector {
                    state: state,
                    on_change: on_interface_change
                }
                DnsInput {
                    state: state,
                    on_settings_change: on_settings_change,
                    on_mode_change: on_mode_change,
                    on_profile_change: on_profile_change,
                    on_new_profile: on_new_profile,
                    on_profile_name_change: on_profile_name_change,
                    on_delete_profile: on_delete_profile,
                }
                ActionButtons {
                    state: state,
                    on_save: on_save,
                    on_apply: on_apply,
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
            {
                let mut write_state = state.write();
                write_state.interfaces = interfaces;
                write_state.selected_interface_index = 0;
            }

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
    }

    refresh_current_dns(state).await;
}

fn change_dns_mode(mut state: Signal<AppState>, mode: DnsMode) {
    let old_mode = state.read().dns_mode;

    if old_mode == mode {
        return;
    }

    let mut write_state = state.write();
    write_state.dns_mode = mode;
    write_state.clear_message();

    if mode == DnsMode::Manual && write_state.config.profiles.is_empty() {
        write_state.create_new_profile();
    } else if mode == DnsMode::Manual
        && write_state.selected_profile_id.is_none()
        && let Some(first) = write_state.config.sorted_profiles().first()
    {
        let first_id = first.id.clone();
        drop(write_state);
        state.write().select_profile(&first_id);
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

async fn save_settings_only(mut state: Signal<AppState>) {
    let validation_result = {
        let read_state = state.read();
        if read_state.dns_mode == DnsMode::Manual {
            read_state.validate_current_settings()
        } else {
            Ok(())
        }
    };

    if let Err(e) = validation_result {
        state.write().set_message(Message::error(e));
        return;
    }

    if state.read().dns_mode == DnsMode::Manual {
        state.write().update_current_profile();
    }

    let config = state.read().config.clone();

    if let Err(e) = save_config(&config) {
        state
            .write()
            .set_message(Message::error(format!("Failed to save config: {}", e)));
    } else {
        state
            .write()
            .set_message(Message::success("Settings saved"));
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
            if state.read().dns_mode == DnsMode::Manual {
                state.write().update_current_profile();
            }

            let config = state.read().config.clone();

            if let Err(e) = save_config(&config) {
                state.write().set_message(Message::error(format!(
                    "Settings applied but failed to save config: {}",
                    e
                )));
            } else {
                state
                    .write()
                    .set_message(Message::success("DNS settings applied successfully"));
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

use crate::dns::{
    AppConfig, CurrentDnsState, DnsMode, DnsProfile, DnsSettings, DohMode, NetworkInterface,
};

#[derive(Clone, Debug)]
pub struct AppState {
    pub interfaces: Vec<NetworkInterface>,
    pub selected_interface_index: usize,
    pub dns_mode: DnsMode,
    pub selected_profile_id: Option<String>,
    pub current_settings: DnsSettings,
    pub current_profile_name: String,
    pub current_dns_state: CurrentDnsState,
    pub config: AppConfig,
    pub message: Option<Message>,
    pub is_loading: bool,
    pub show_delete_confirm: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageLevel {
    Success,
    Warning,
    Error,
}

#[derive(Clone, Debug)]
pub struct Message {
    pub text: String,
    pub level: MessageLevel,
}

impl Message {
    pub fn success(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            level: MessageLevel::Success,
        }
    }

    pub fn error(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            level: MessageLevel::Error,
        }
    }

    pub fn warning(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            level: MessageLevel::Warning,
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            interfaces: Vec::new(),
            selected_interface_index: 0,
            dns_mode: DnsMode::Automatic,
            selected_profile_id: None,
            current_settings: DnsSettings::new(),
            current_profile_name: String::new(),
            current_dns_state: CurrentDnsState::new(),
            config: AppConfig::new(),
            message: None,
            is_loading: false,
            show_delete_confirm: false,
        }
    }

    pub fn selected_interface(&self) -> Option<&NetworkInterface> {
        self.interfaces.get(self.selected_interface_index)
    }

    pub fn set_message(&mut self, message: Message) {
        self.message = Some(message);
    }

    pub fn clear_message(&mut self) {
        self.message = None;
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.is_loading = loading;
    }

    pub fn sorted_profiles(&self) -> Vec<&DnsProfile> {
        self.config.sorted_profiles()
    }

    pub fn select_profile(&mut self, id: &str) {
        if let Some(profile) = self.config.find_profile(id) {
            self.selected_profile_id = Some(id.to_string());
            self.current_settings = profile.settings.clone();
            self.current_profile_name = profile.name.clone();
        }
    }

    pub fn create_new_profile(&mut self) -> String {
        let mut name = "New Profile".to_string();
        let mut counter = 1;
        while self.config.profiles.iter().any(|p| p.name == name) {
            counter += 1;
            name = format!("New Profile {}", counter);
        }

        let profile = DnsProfile::new(name);
        let id = profile.id.clone();
        self.config.add_profile(profile);
        self.select_profile(&id);
        id
    }

    pub fn update_current_profile(&mut self) {
        let id = match &self.selected_profile_id {
            Some(id) => id.clone(),
            None => return,
        };
        if let Some(profile) = self.config.find_profile_mut(&id) {
            profile.name = self.current_profile_name.clone();
            profile.settings = self.current_settings.clone();
        }
    }

    pub fn delete_current_profile(&mut self) {
        if let Some(id) = self.selected_profile_id.take() {
            self.config.remove_profile(&id);
            self.current_settings = DnsSettings::new();
            self.current_profile_name = String::new();

            if let Some(first) = self.config.sorted_profiles().first() {
                let first_id = first.id.clone();
                self.select_profile(&first_id);
            } else {
                self.dns_mode = DnsMode::Automatic;
            }
        }
    }

    pub fn is_profile_name_duplicate(&self, name: &str, exclude_id: Option<&str>) -> bool {
        self.config.profiles.iter().any(|p| {
            p.name.to_lowercase() == name.to_lowercase() && exclude_id.is_none_or(|id| p.id != id)
        })
    }

    pub fn validate_current_settings(&self) -> Result<(), String> {
        if self.dns_mode == DnsMode::Automatic {
            return Ok(());
        }

        if self.selected_profile_id.is_none() {
            return Err("No profile selected".to_string());
        }

        if self.dns_mode == DnsMode::Manual {
            if self.current_profile_name.trim().is_empty() {
                return Err("Profile name cannot be empty".to_string());
            }

            if let Some(ref id) = self.selected_profile_id
                && self.is_profile_name_duplicate(&self.current_profile_name, Some(id))
            {
                return Err("A profile with this name already exists".to_string());
            }
        }

        let ipv4_entry = &self.current_settings.ipv4;
        let ipv6_entry = &self.current_settings.ipv6;

        if ipv4_entry.enabled {
            if ipv4_entry.primary.address.is_empty() {
                return Err("IPv4 primary DNS is required when enabled".to_string());
            }
            if !crate::dns::validate_ipv4(&ipv4_entry.primary.address) {
                return Err("Invalid IPv4 primary DNS address".to_string());
            }
            if !ipv4_entry.secondary.address.is_empty()
                && !crate::dns::validate_ipv4(&ipv4_entry.secondary.address)
            {
                return Err("Invalid IPv4 secondary DNS address".to_string());
            }
            if ipv4_entry.primary.doh_mode == DohMode::On {
                if ipv4_entry.primary.doh_template.is_empty() {
                    return Err(
                        "IPv4 primary DoH template URL is required when DoH is enabled".to_string(),
                    );
                }
                if !crate::dns::validate_doh_template(&ipv4_entry.primary.doh_template) {
                    return Err("Invalid IPv4 primary DoH template URL".to_string());
                }
            }
            if ipv4_entry.secondary.doh_mode == DohMode::On {
                if ipv4_entry.secondary.address.is_empty() {
                    return Err(
                        "IPv4 secondary DNS address is required when DoH is enabled".to_string()
                    );
                }
                if ipv4_entry.secondary.doh_template.is_empty() {
                    return Err(
                        "IPv4 secondary DoH template URL is required when DoH is enabled"
                            .to_string(),
                    );
                }
                if !crate::dns::validate_doh_template(&ipv4_entry.secondary.doh_template) {
                    return Err("Invalid IPv4 secondary DoH template URL".to_string());
                }
            }
        }

        if ipv6_entry.enabled {
            if ipv6_entry.primary.address.is_empty() {
                return Err("IPv6 primary DNS is required when enabled".to_string());
            }
            if !crate::dns::validate_ipv6(&ipv6_entry.primary.address) {
                return Err("Invalid IPv6 primary DNS address".to_string());
            }
            if !ipv6_entry.secondary.address.is_empty()
                && !crate::dns::validate_ipv6(&ipv6_entry.secondary.address)
            {
                return Err("Invalid IPv6 secondary DNS address".to_string());
            }
            if ipv6_entry.primary.doh_mode == DohMode::On {
                if ipv6_entry.primary.doh_template.is_empty() {
                    return Err(
                        "IPv6 primary DoH template URL is required when DoH is enabled".to_string(),
                    );
                }
                if !crate::dns::validate_doh_template(&ipv6_entry.primary.doh_template) {
                    return Err("Invalid IPv6 primary DoH template URL".to_string());
                }
            }
            if ipv6_entry.secondary.doh_mode == DohMode::On {
                if ipv6_entry.secondary.address.is_empty() {
                    return Err(
                        "IPv6 secondary DNS address is required when DoH is enabled".to_string()
                    );
                }
                if ipv6_entry.secondary.doh_template.is_empty() {
                    return Err(
                        "IPv6 secondary DoH template URL is required when DoH is enabled"
                            .to_string(),
                    );
                }
                if !crate::dns::validate_doh_template(&ipv6_entry.secondary.doh_template) {
                    return Err("Invalid IPv6 secondary DoH template URL".to_string());
                }
            }
        }

        Ok(())
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dns::{DnsEntry, DnsServerEntry, DohMode, NetworkInterface};

    fn create_test_interface(name: &str, index: u32) -> NetworkInterface {
        NetworkInterface {
            name: name.to_string(),
            interface_index: index,
            interface_guid: format!("{{GUID-{}}}", index),
            has_ipv4: true,
            has_ipv6: true,
        }
    }

    fn create_valid_ipv4_settings() -> DnsEntry {
        DnsEntry {
            enabled: true,
            primary: DnsServerEntry {
                address: "8.8.8.8".to_string(),
                doh_mode: DohMode::Off,
                doh_template: String::new(),
                allow_fallback: true,
            },
            secondary: DnsServerEntry::default(),
        }
    }

    fn create_valid_ipv6_settings() -> DnsEntry {
        DnsEntry {
            enabled: true,
            primary: DnsServerEntry {
                address: "2001:4860:4860::8888".to_string(),
                doh_mode: DohMode::Off,
                doh_template: String::new(),
                allow_fallback: true,
            },
            secondary: DnsServerEntry::default(),
        }
    }

    #[test]
    fn test_message_success() {
        let msg = Message::success("Success message");
        assert_eq!(msg.text, "Success message");
        assert_eq!(msg.level, MessageLevel::Success);
    }

    #[test]
    fn test_message_error() {
        let msg = Message::error("Error message");
        assert_eq!(msg.text, "Error message");
        assert_eq!(msg.level, MessageLevel::Error);
    }

    #[test]
    fn test_message_warning() {
        let msg = Message::warning("Warning message");
        assert_eq!(msg.text, "Warning message");
        assert_eq!(msg.level, MessageLevel::Warning);
    }

    #[test]
    fn test_app_state_new() {
        let state = AppState::new();
        assert_eq!(state.interfaces.len(), 0);
        assert_eq!(state.selected_interface_index, 0);
        assert_eq!(state.dns_mode, DnsMode::Automatic);
        assert!(state.selected_profile_id.is_none());
        assert!(!state.current_settings.ipv4.enabled);
        assert!(!state.current_settings.ipv6.enabled);
        assert_eq!(state.current_profile_name, "");
        assert_eq!(state.config.profiles.len(), 0);
        assert!(state.message.is_none());
        assert!(!state.is_loading);
        assert!(!state.show_delete_confirm);
    }

    #[test]
    fn test_app_state_selected_interface_when_empty() {
        let state = AppState::new();
        assert!(state.selected_interface().is_none());
    }

    #[test]
    fn test_app_state_selected_interface_when_in_range() {
        let mut state = AppState::new();
        state.interfaces.push(create_test_interface("Ethernet", 1));
        state.interfaces.push(create_test_interface("WiFi", 2));
        state.selected_interface_index = 1;

        let selected = state.selected_interface();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name, "WiFi");
    }

    #[test]
    fn test_app_state_set_message() {
        let mut state = AppState::new();
        state.set_message(Message::success("Test"));
        assert!(state.message.is_some());
        assert_eq!(state.message.as_ref().unwrap().text, "Test");
        assert_eq!(state.message.as_ref().unwrap().level, MessageLevel::Success);
    }

    #[test]
    fn test_app_state_clear_message() {
        let mut state = AppState::new();
        state.set_message(Message::success("Test"));
        state.clear_message();
        assert!(state.message.is_none());
    }

    #[test]
    fn test_app_state_set_loading_true() {
        let mut state = AppState::new();
        state.set_loading(true);
        assert!(state.is_loading);
    }

    #[test]
    fn test_app_state_set_loading_false() {
        let mut state = AppState::new();
        state.set_loading(true);
        state.set_loading(false);
        assert!(!state.is_loading);
    }

    #[test]
    fn test_app_state_sorted_profiles() {
        let mut state = AppState::new();
        state
            .config
            .add_profile(DnsProfile::new("Zebra".to_string()));
        state
            .config
            .add_profile(DnsProfile::new("Apple".to_string()));

        let sorted = state.sorted_profiles();
        assert_eq!(sorted.len(), 2);
        assert_eq!(sorted[0].name, "Apple");
        assert_eq!(sorted[1].name, "Zebra");
    }

    #[test]
    fn test_app_state_select_profile_when_exists() {
        let mut state = AppState::new();
        let profile = DnsProfile::new("Test Profile".to_string());
        let id = profile.id.clone();
        state.config.add_profile(profile);

        state.select_profile(&id);
        assert_eq!(state.selected_profile_id, Some(id));
        assert_eq!(state.current_profile_name, "Test Profile");
    }

    #[test]
    fn test_app_state_select_profile_when_not_exists() {
        let mut state = AppState::new();
        state.select_profile("non-existent-id");
        assert!(state.selected_profile_id.is_none());
    }

    #[test]
    fn test_app_state_create_new_profile_first() {
        let mut state = AppState::new();
        let id = state.create_new_profile();

        assert_eq!(state.selected_profile_id, Some(id.clone()));
        assert_eq!(state.current_profile_name, "New Profile");
        assert_eq!(state.config.profiles.len(), 1);
    }

    #[test]
    fn test_app_state_create_new_profile_with_duplicates() {
        let mut state = AppState::new();
        state
            .config
            .add_profile(DnsProfile::new("New Profile".to_string()));
        state
            .config
            .add_profile(DnsProfile::new("New Profile 2".to_string()));

        let id = state.create_new_profile();
        assert_eq!(state.current_profile_name, "New Profile 3");
        assert_eq!(state.config.profiles.len(), 3);
        assert_eq!(state.selected_profile_id, Some(id));
    }

    #[test]
    fn test_app_state_update_current_profile_when_selected() {
        let mut state = AppState::new();
        let profile = DnsProfile::new("Original Name".to_string());
        let id = profile.id.clone();
        state.config.add_profile(profile);
        state.select_profile(&id);

        state.current_profile_name = "Updated Name".to_string();
        state.current_settings.ipv4 = create_valid_ipv4_settings();
        state.update_current_profile();

        let updated = state.config.find_profile(&id).unwrap();
        assert_eq!(updated.name, "Updated Name");
        assert!(updated.settings.ipv4.enabled);
    }

    #[test]
    fn test_app_state_update_current_profile_when_not_selected() {
        let mut state = AppState::new();
        state.current_profile_name = "Test".to_string();
        state.update_current_profile();
        assert_eq!(state.config.profiles.len(), 0);
    }

    #[test]
    fn test_app_state_delete_current_profile_with_other_profiles() {
        let mut state = AppState::new();
        let profile1 = DnsProfile::new("Profile 1".to_string());
        let id1 = profile1.id.clone();
        let profile2 = DnsProfile::new("Profile 2".to_string());
        let id2 = profile2.id.clone();
        state.config.add_profile(profile1);
        state.config.add_profile(profile2);
        state.select_profile(&id1);

        state.delete_current_profile();

        assert_eq!(state.config.profiles.len(), 1);
        assert_eq!(state.selected_profile_id, Some(id2));
        assert_eq!(state.current_profile_name, "Profile 2");
    }

    #[test]
    fn test_app_state_delete_current_profile_last_profile() {
        let mut state = AppState::new();
        let profile = DnsProfile::new("Last Profile".to_string());
        let id = profile.id.clone();
        state.config.add_profile(profile);
        state.select_profile(&id);
        state.dns_mode = DnsMode::Manual;

        state.delete_current_profile();

        assert_eq!(state.config.profiles.len(), 0);
        assert!(state.selected_profile_id.is_none());
        assert_eq!(state.dns_mode, DnsMode::Automatic);
    }

    #[test]
    fn test_app_state_delete_current_profile_when_not_selected() {
        let mut state = AppState::new();
        state
            .config
            .add_profile(DnsProfile::new("Profile".to_string()));
        let initial_count = state.config.profiles.len();

        state.delete_current_profile();

        assert_eq!(state.config.profiles.len(), initial_count);
    }

    #[test]
    fn test_app_state_is_profile_name_duplicate_when_duplicate() {
        let mut state = AppState::new();
        state
            .config
            .add_profile(DnsProfile::new("Test Profile".to_string()));

        assert!(state.is_profile_name_duplicate("Test Profile", None));
    }

    #[test]
    fn test_app_state_is_profile_name_duplicate_when_not_duplicate() {
        let mut state = AppState::new();
        state
            .config
            .add_profile(DnsProfile::new("Test Profile".to_string()));

        assert!(!state.is_profile_name_duplicate("Other Profile", None));
    }

    #[test]
    fn test_app_state_is_profile_name_duplicate_case_insensitive() {
        let mut state = AppState::new();
        state
            .config
            .add_profile(DnsProfile::new("Test Profile".to_string()));

        assert!(state.is_profile_name_duplicate("test profile", None));
        assert!(state.is_profile_name_duplicate("TEST PROFILE", None));
    }

    #[test]
    fn test_app_state_is_profile_name_duplicate_exclude_self() {
        let mut state = AppState::new();
        let profile = DnsProfile::new("Test Profile".to_string());
        let id = profile.id.clone();
        state.config.add_profile(profile);

        assert!(!state.is_profile_name_duplicate("Test Profile", Some(&id)));
    }

    #[test]
    fn test_app_state_validate_current_settings_automatic_mode() {
        let state = AppState::new();
        assert!(state.validate_current_settings().is_ok());
    }

    #[test]
    fn test_app_state_validate_current_settings_no_profile_selected() {
        let mut state = AppState::new();
        state.dns_mode = DnsMode::Manual;
        let result = state.validate_current_settings();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No profile selected");
    }

    #[test]
    fn test_app_state_validate_current_settings_empty_profile_name() {
        let mut state = AppState::new();
        state.dns_mode = DnsMode::Manual;
        state.selected_profile_id = Some("test-id".to_string());
        state.current_profile_name = "".to_string();

        let result = state.validate_current_settings();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Profile name cannot be empty");
    }

    #[test]
    fn test_app_state_validate_current_settings_duplicate_profile_name() {
        let mut state = AppState::new();
        state
            .config
            .add_profile(DnsProfile::new("Existing".to_string()));
        let profile = DnsProfile::new("Test".to_string());
        let id = profile.id.clone();
        state.config.add_profile(profile);
        state.select_profile(&id);
        state.dns_mode = DnsMode::Manual;
        state.current_profile_name = "Existing".to_string();

        let result = state.validate_current_settings();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "A profile with this name already exists"
        );
    }

    #[test]
    fn test_app_state_validate_current_settings_ipv4_enabled_empty_primary() {
        let mut state = AppState::new();
        let profile = DnsProfile::new("Test".to_string());
        let id = profile.id.clone();
        state.config.add_profile(profile);
        state.select_profile(&id);
        state.dns_mode = DnsMode::Manual;
        state.current_settings.ipv4.enabled = true;

        let result = state.validate_current_settings();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "IPv4 primary DNS is required when enabled"
        );
    }

    #[test]
    fn test_app_state_validate_current_settings_ipv4_enabled_invalid_primary() {
        let mut state = AppState::new();
        let profile = DnsProfile::new("Test".to_string());
        let id = profile.id.clone();
        state.config.add_profile(profile);
        state.select_profile(&id);
        state.dns_mode = DnsMode::Manual;
        state.current_settings.ipv4.enabled = true;
        state.current_settings.ipv4.primary.address = "invalid".to_string();

        let result = state.validate_current_settings();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid IPv4 primary DNS address");
    }

    #[test]
    fn test_app_state_validate_current_settings_ipv4_enabled_invalid_secondary() {
        let mut state = AppState::new();
        let profile = DnsProfile::new("Test".to_string());
        let id = profile.id.clone();
        state.config.add_profile(profile);
        state.select_profile(&id);
        state.dns_mode = DnsMode::Manual;
        state.current_settings.ipv4 = create_valid_ipv4_settings();
        state.current_settings.ipv4.secondary.address = "invalid".to_string();

        let result = state.validate_current_settings();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid IPv4 secondary DNS address");
    }

    #[test]
    fn test_app_state_validate_current_settings_ipv4_doh_on_empty_template() {
        let mut state = AppState::new();
        let profile = DnsProfile::new("Test".to_string());
        let id = profile.id.clone();
        state.config.add_profile(profile);
        state.select_profile(&id);
        state.dns_mode = DnsMode::Manual;
        state.current_settings.ipv4 = create_valid_ipv4_settings();
        state.current_settings.ipv4.primary.doh_mode = DohMode::On;

        let result = state.validate_current_settings();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "IPv4 primary DoH template URL is required when DoH is enabled"
        );
    }

    #[test]
    fn test_app_state_validate_current_settings_ipv4_doh_on_invalid_template() {
        let mut state = AppState::new();
        let profile = DnsProfile::new("Test".to_string());
        let id = profile.id.clone();
        state.config.add_profile(profile);
        state.select_profile(&id);
        state.dns_mode = DnsMode::Manual;
        state.current_settings.ipv4 = create_valid_ipv4_settings();
        state.current_settings.ipv4.primary.doh_mode = DohMode::On;
        state.current_settings.ipv4.primary.doh_template = "invalid".to_string();

        let result = state.validate_current_settings();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid IPv4 primary DoH template URL");
    }

    #[test]
    fn test_app_state_validate_current_settings_ipv4_secondary_doh_on_empty_template() {
        let mut state = AppState::new();
        let profile = DnsProfile::new("Test".to_string());
        let id = profile.id.clone();
        state.config.add_profile(profile);
        state.select_profile(&id);
        state.dns_mode = DnsMode::Manual;
        state.current_settings.ipv4 = create_valid_ipv4_settings();
        state.current_settings.ipv4.secondary.address = "8.8.4.4".to_string();
        state.current_settings.ipv4.secondary.doh_mode = DohMode::On;

        let result = state.validate_current_settings();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "IPv4 secondary DoH template URL is required when DoH is enabled"
        );
    }

    #[test]
    fn test_app_state_validate_current_settings_ipv4_secondary_doh_on_invalid_template() {
        let mut state = AppState::new();
        let profile = DnsProfile::new("Test".to_string());
        let id = profile.id.clone();
        state.config.add_profile(profile);
        state.select_profile(&id);
        state.dns_mode = DnsMode::Manual;
        state.current_settings.ipv4 = create_valid_ipv4_settings();
        state.current_settings.ipv4.secondary.address = "8.8.4.4".to_string();
        state.current_settings.ipv4.secondary.doh_mode = DohMode::On;
        state.current_settings.ipv4.secondary.doh_template = "invalid".to_string();

        let result = state.validate_current_settings();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Invalid IPv4 secondary DoH template URL"
        );
    }

    #[test]
    fn test_app_state_validate_current_settings_ipv6_enabled_empty_primary() {
        let mut state = AppState::new();
        let profile = DnsProfile::new("Test".to_string());
        let id = profile.id.clone();
        state.config.add_profile(profile);
        state.select_profile(&id);
        state.dns_mode = DnsMode::Manual;
        state.current_settings.ipv6.enabled = true;

        let result = state.validate_current_settings();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "IPv6 primary DNS is required when enabled"
        );
    }

    #[test]
    fn test_app_state_validate_current_settings_ipv6_enabled_invalid_primary() {
        let mut state = AppState::new();
        let profile = DnsProfile::new("Test".to_string());
        let id = profile.id.clone();
        state.config.add_profile(profile);
        state.select_profile(&id);
        state.dns_mode = DnsMode::Manual;
        state.current_settings.ipv6.enabled = true;
        state.current_settings.ipv6.primary.address = "invalid".to_string();

        let result = state.validate_current_settings();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid IPv6 primary DNS address");
    }

    #[test]
    fn test_app_state_validate_current_settings_ipv6_enabled_invalid_secondary() {
        let mut state = AppState::new();
        let profile = DnsProfile::new("Test".to_string());
        let id = profile.id.clone();
        state.config.add_profile(profile);
        state.select_profile(&id);
        state.dns_mode = DnsMode::Manual;
        state.current_settings.ipv6 = create_valid_ipv6_settings();
        state.current_settings.ipv6.secondary.address = "invalid".to_string();

        let result = state.validate_current_settings();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid IPv6 secondary DNS address");
    }

    #[test]
    fn test_app_state_validate_current_settings_ipv6_doh_on_empty_template() {
        let mut state = AppState::new();
        let profile = DnsProfile::new("Test".to_string());
        let id = profile.id.clone();
        state.config.add_profile(profile);
        state.select_profile(&id);
        state.dns_mode = DnsMode::Manual;
        state.current_settings.ipv6 = create_valid_ipv6_settings();
        state.current_settings.ipv6.primary.doh_mode = DohMode::On;

        let result = state.validate_current_settings();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "IPv6 primary DoH template URL is required when DoH is enabled"
        );
    }

    #[test]
    fn test_app_state_validate_current_settings_ipv6_doh_on_invalid_template() {
        let mut state = AppState::new();
        let profile = DnsProfile::new("Test".to_string());
        let id = profile.id.clone();
        state.config.add_profile(profile);
        state.select_profile(&id);
        state.dns_mode = DnsMode::Manual;
        state.current_settings.ipv6 = create_valid_ipv6_settings();
        state.current_settings.ipv6.primary.doh_mode = DohMode::On;
        state.current_settings.ipv6.primary.doh_template = "invalid".to_string();

        let result = state.validate_current_settings();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid IPv6 primary DoH template URL");
    }

    #[test]
    fn test_app_state_validate_current_settings_ipv6_secondary_doh_on_empty_template() {
        let mut state = AppState::new();
        let profile = DnsProfile::new("Test".to_string());
        let id = profile.id.clone();
        state.config.add_profile(profile);
        state.select_profile(&id);
        state.dns_mode = DnsMode::Manual;
        state.current_settings.ipv6 = create_valid_ipv6_settings();
        state.current_settings.ipv6.secondary.address = "2001:4860:4860::8844".to_string();
        state.current_settings.ipv6.secondary.doh_mode = DohMode::On;

        let result = state.validate_current_settings();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "IPv6 secondary DoH template URL is required when DoH is enabled"
        );
    }

    #[test]
    fn test_app_state_validate_current_settings_ipv6_secondary_doh_on_invalid_template() {
        let mut state = AppState::new();
        let profile = DnsProfile::new("Test".to_string());
        let id = profile.id.clone();
        state.config.add_profile(profile);
        state.select_profile(&id);
        state.dns_mode = DnsMode::Manual;
        state.current_settings.ipv6 = create_valid_ipv6_settings();
        state.current_settings.ipv6.secondary.address = "2001:4860:4860::8844".to_string();
        state.current_settings.ipv6.secondary.doh_mode = DohMode::On;
        state.current_settings.ipv6.secondary.doh_template = "invalid".to_string();

        let result = state.validate_current_settings();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Invalid IPv6 secondary DoH template URL"
        );
    }

    #[test]
    fn test_app_state_validate_current_settings_valid_configuration() {
        let mut state = AppState::new();
        let profile = DnsProfile::new("Test".to_string());
        let id = profile.id.clone();
        state.config.add_profile(profile);
        state.select_profile(&id);
        state.dns_mode = DnsMode::Manual;
        state.current_settings.ipv4 = create_valid_ipv4_settings();
        state.current_settings.ipv6 = create_valid_ipv6_settings();

        let result = state.validate_current_settings();
        assert!(result.is_ok());
    }

    #[test]
    fn test_app_state_default() {
        let default_state = AppState::default();
        let new_state = AppState::new();

        assert_eq!(default_state.interfaces.len(), new_state.interfaces.len());
        assert_eq!(
            default_state.selected_interface_index,
            new_state.selected_interface_index
        );
        assert_eq!(default_state.dns_mode, new_state.dns_mode);
        assert_eq!(
            default_state.selected_profile_id,
            new_state.selected_profile_id
        );
        assert_eq!(
            default_state.current_profile_name,
            new_state.current_profile_name
        );
        assert_eq!(
            default_state.config.profiles.len(),
            new_state.config.profiles.len()
        );
        assert_eq!(default_state.message.is_none(), new_state.message.is_none());
        assert_eq!(default_state.is_loading, new_state.is_loading);
        assert_eq!(
            default_state.show_delete_confirm,
            new_state.show_delete_confirm
        );
    }
}

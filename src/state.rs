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

#[derive(Clone, Debug)]
pub struct Message {
    pub text: String,
    pub is_error: bool,
}

impl Message {
    pub fn success(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            is_error: false,
        }
    }

    pub fn error(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            is_error: true,
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

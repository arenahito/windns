use crate::dns::{AppConfig, CurrentDnsState, DnsMode, DnsSettings, DohMode, NetworkInterface};

#[derive(Clone, Debug)]
pub struct AppState {
    pub interfaces: Vec<NetworkInterface>,
    pub selected_interface_index: usize,
    pub dns_mode: DnsMode,
    pub current_settings: DnsSettings,
    pub current_dns_state: CurrentDnsState,
    pub config: AppConfig,
    pub message: Option<Message>,
    pub is_loading: bool,
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
            current_settings: DnsSettings::new(),
            current_dns_state: CurrentDnsState::new(),
            config: AppConfig::new(),
            message: None,
            is_loading: false,
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

    pub fn load_settings_for_mode(&mut self, mode: DnsMode) {
        if let Some(interface) = self.selected_interface() {
            if let Some(config) = self.config.find_interface(&interface.interface_guid) {
                self.current_settings = match mode {
                    DnsMode::Manual => config.manual_settings.clone(),
                    DnsMode::Automatic => DnsSettings::new(),
                };
            } else {
                self.current_settings = DnsSettings::new();
            }
        }
    }

    pub fn save_settings_for_mode(&mut self, mode: DnsMode) {
        if let Some(interface) = self.selected_interface() {
            let guid = interface.interface_guid.clone();
            let name = interface.name.clone();

            let config = self.config.ensure_interface(guid, name);

            match mode {
                DnsMode::Manual => {
                    config.manual_settings = self.current_settings.clone();
                }
                DnsMode::Automatic => {}
            }
        }
    }

    pub fn validate_current_settings(&self) -> Result<(), String> {
        if self.dns_mode == DnsMode::Automatic {
            return Ok(());
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

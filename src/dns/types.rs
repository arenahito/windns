use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, Debug)]
pub enum DnsMode {
    #[default]
    Automatic,
    Manual,
}

impl DnsMode {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            DnsMode::Automatic => "Automatic",
            DnsMode::Manual => "Manual",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum AddressFamily {
    IPv4,
    IPv6,
}

impl AddressFamily {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            AddressFamily::IPv4 => "IPv4",
            AddressFamily::IPv6 => "IPv6",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, Debug)]
pub enum DohMode {
    #[default]
    Off,
    On,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct DnsServerEntry {
    pub address: String,
    pub doh_mode: DohMode,
    pub doh_template: String,
    pub allow_fallback: bool,
}

impl Default for DnsServerEntry {
    fn default() -> Self {
        Self {
            address: String::new(),
            doh_mode: DohMode::Off,
            doh_template: String::new(),
            allow_fallback: true,
        }
    }
}

impl DnsServerEntry {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct NetworkInterface {
    pub name: String,
    pub interface_index: u32,
    pub interface_guid: String,
    pub has_ipv4: bool,
    pub has_ipv6: bool,
}

impl NetworkInterface {
    pub fn display_name(&self) -> String {
        format!("{} ({})", self.name, self.interface_index)
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Default, Debug)]
pub struct DnsEntry {
    pub enabled: bool,
    pub primary: DnsServerEntry,
    pub secondary: DnsServerEntry,
}

impl DnsEntry {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn is_valid(&self) -> bool {
        if !self.enabled {
            return true;
        }
        !self.primary.address.is_empty()
    }

    pub fn get_addresses(&self) -> Vec<String> {
        let mut addresses = Vec::new();
        if !self.primary.address.is_empty() {
            addresses.push(self.primary.address.clone());
        }
        if !self.secondary.address.is_empty() {
            addresses.push(self.secondary.address.clone());
        }
        addresses
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Default, Debug)]
pub struct DnsSettings {
    pub ipv4: DnsEntry,
    pub ipv6: DnsEntry,
}

impl DnsSettings {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct DnsProfile {
    pub id: String,
    pub name: String,
    pub settings: DnsSettings,
}

impl DnsProfile {
    pub fn new(name: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            settings: DnsSettings::new(),
        }
    }
}

/// Window state with position in physical pixels and size in logical pixels.
/// Physical position ensures exact screen location restoration.
/// Logical size ensures consistent visual appearance across DPI settings.
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct WindowState {
    /// X position in physical pixels
    pub x: i32,
    /// Y position in physical pixels
    pub y: i32,
    /// Width in logical pixels
    pub width: u32,
    /// Height in logical pixels
    pub height: u32,
    pub maximized: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            x: 100,
            y: 100,
            width: 850,
            height: 700,
            maximized: false,
        }
    }
}

impl WindowState {
    pub const MIN_WIDTH: u32 = 400;
    pub const MIN_HEIGHT: u32 = 300;
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Default, Debug)]
pub struct AppConfig {
    #[serde(default)]
    pub profiles: Vec<DnsProfile>,
    #[serde(default)]
    pub window: Option<WindowState>,
}

impl AppConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn find_profile(&self, id: &str) -> Option<&DnsProfile> {
        self.profiles.iter().find(|p| p.id == id)
    }

    pub fn find_profile_mut(&mut self, id: &str) -> Option<&mut DnsProfile> {
        self.profiles.iter_mut().find(|p| p.id == id)
    }

    pub fn add_profile(&mut self, profile: DnsProfile) {
        self.profiles.push(profile);
    }

    pub fn remove_profile(&mut self, id: &str) -> bool {
        if let Some(pos) = self.profiles.iter().position(|p| p.id == id) {
            self.profiles.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn sorted_profiles(&self) -> Vec<&DnsProfile> {
        let mut profiles: Vec<_> = self.profiles.iter().collect();
        profiles.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        profiles
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CurrentDnsState {
    pub ipv4: Vec<String>,
    pub ipv6: Vec<String>,
}

impl CurrentDnsState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_display(&self, family: AddressFamily) -> String {
        let addresses = match family {
            AddressFamily::IPv4 => &self.ipv4,
            AddressFamily::IPv6 => &self.ipv6,
        };

        if addresses.is_empty() {
            "Automatic".to_string()
        } else {
            addresses.join(", ")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dns_mode_as_str_automatic() {
        assert_eq!(DnsMode::Automatic.as_str(), "Automatic");
    }

    #[test]
    fn test_dns_mode_as_str_manual() {
        assert_eq!(DnsMode::Manual.as_str(), "Manual");
    }

    #[test]
    fn test_address_family_as_str_ipv4() {
        assert_eq!(AddressFamily::IPv4.as_str(), "IPv4");
    }

    #[test]
    fn test_address_family_as_str_ipv6() {
        assert_eq!(AddressFamily::IPv6.as_str(), "IPv6");
    }

    #[test]
    fn test_dns_server_entry_new() {
        let entry = DnsServerEntry::new();
        assert_eq!(entry.address, "");
        assert_eq!(entry.doh_mode, DohMode::Off);
        assert_eq!(entry.doh_template, "");
        assert!(entry.allow_fallback);
    }

    #[test]
    fn test_network_interface_display_name() {
        let interface = NetworkInterface {
            name: "Ethernet".to_string(),
            interface_index: 12,
            interface_guid: "{GUID}".to_string(),
            has_ipv4: true,
            has_ipv6: false,
        };
        assert_eq!(interface.display_name(), "Ethernet (12)");
    }

    #[test]
    fn test_dns_entry_new() {
        let entry = DnsEntry::new();
        assert!(!entry.enabled);
        assert_eq!(entry.primary.address, "");
        assert_eq!(entry.secondary.address, "");
    }

    #[test]
    fn test_dns_entry_is_valid_when_disabled() {
        let entry = DnsEntry {
            enabled: false,
            primary: DnsServerEntry::default(),
            secondary: DnsServerEntry::default(),
        };
        assert!(entry.is_valid());
    }

    #[test]
    fn test_dns_entry_is_valid_when_enabled_with_empty_primary() {
        let entry = DnsEntry {
            enabled: true,
            primary: DnsServerEntry::default(),
            secondary: DnsServerEntry::default(),
        };
        assert!(!entry.is_valid());
    }

    #[test]
    fn test_dns_entry_is_valid_when_enabled_with_primary() {
        let entry = DnsEntry {
            enabled: true,
            primary: DnsServerEntry {
                address: "8.8.8.8".to_string(),
                ..Default::default()
            },
            secondary: DnsServerEntry::default(),
        };
        assert!(entry.is_valid());
    }

    #[test]
    fn test_dns_entry_get_addresses_when_both_empty() {
        let entry = DnsEntry::new();
        assert_eq!(entry.get_addresses(), Vec::<String>::new());
    }

    #[test]
    fn test_dns_entry_get_addresses_when_primary_only() {
        let entry = DnsEntry {
            enabled: true,
            primary: DnsServerEntry {
                address: "8.8.8.8".to_string(),
                ..Default::default()
            },
            secondary: DnsServerEntry::default(),
        };
        assert_eq!(entry.get_addresses(), vec!["8.8.8.8"]);
    }

    #[test]
    fn test_dns_entry_get_addresses_when_both_set() {
        let entry = DnsEntry {
            enabled: true,
            primary: DnsServerEntry {
                address: "8.8.8.8".to_string(),
                ..Default::default()
            },
            secondary: DnsServerEntry {
                address: "8.8.4.4".to_string(),
                ..Default::default()
            },
        };
        assert_eq!(entry.get_addresses(), vec!["8.8.8.8", "8.8.4.4"]);
    }

    #[test]
    fn test_dns_settings_new() {
        let settings = DnsSettings::new();
        assert!(!settings.ipv4.enabled);
        assert!(!settings.ipv6.enabled);
    }

    #[test]
    fn test_dns_profile_new() {
        let profile = DnsProfile::new("Test Profile".to_string());
        assert_eq!(profile.name, "Test Profile");
        assert!(!profile.id.is_empty());
        assert!(!profile.settings.ipv4.enabled);
        assert!(!profile.settings.ipv6.enabled);
    }

    #[test]
    fn test_app_config_new() {
        let config = AppConfig::new();
        assert_eq!(config.profiles.len(), 0);
    }

    #[test]
    fn test_app_config_find_profile_found() {
        let mut config = AppConfig::new();
        let profile = DnsProfile::new("Test".to_string());
        let id = profile.id.clone();
        config.add_profile(profile);

        let found = config.find_profile(&id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Test");
    }

    #[test]
    fn test_app_config_find_profile_not_found() {
        let config = AppConfig::new();
        let found = config.find_profile("non-existent-id");
        assert!(found.is_none());
    }

    #[test]
    fn test_app_config_find_profile_mut_found() {
        let mut config = AppConfig::new();
        let profile = DnsProfile::new("Test".to_string());
        let id = profile.id.clone();
        config.add_profile(profile);

        let found = config.find_profile_mut(&id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Test");
    }

    #[test]
    fn test_app_config_find_profile_mut_not_found() {
        let mut config = AppConfig::new();
        let found = config.find_profile_mut("non-existent-id");
        assert!(found.is_none());
    }

    #[test]
    fn test_app_config_add_profile() {
        let mut config = AppConfig::new();
        let profile = DnsProfile::new("Test".to_string());
        config.add_profile(profile);
        assert_eq!(config.profiles.len(), 1);
    }

    #[test]
    fn test_app_config_remove_profile_success() {
        let mut config = AppConfig::new();
        let profile = DnsProfile::new("Test".to_string());
        let id = profile.id.clone();
        config.add_profile(profile);

        let result = config.remove_profile(&id);
        assert!(result);
        assert_eq!(config.profiles.len(), 0);
    }

    #[test]
    fn test_app_config_remove_profile_not_found() {
        let mut config = AppConfig::new();
        let result = config.remove_profile("non-existent-id");
        assert!(!result);
    }

    #[test]
    fn test_app_config_sorted_profiles_empty() {
        let config = AppConfig::new();
        let sorted = config.sorted_profiles();
        assert_eq!(sorted.len(), 0);
    }

    #[test]
    fn test_app_config_sorted_profiles_sorted_by_name() {
        let mut config = AppConfig::new();
        config.add_profile(DnsProfile::new("Zebra".to_string()));
        config.add_profile(DnsProfile::new("apple".to_string()));
        config.add_profile(DnsProfile::new("Banana".to_string()));

        let sorted = config.sorted_profiles();
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].name, "apple");
        assert_eq!(sorted[1].name, "Banana");
        assert_eq!(sorted[2].name, "Zebra");
    }

    #[test]
    fn test_current_dns_state_new() {
        let state = CurrentDnsState::new();
        assert_eq!(state.ipv4.len(), 0);
        assert_eq!(state.ipv6.len(), 0);
    }

    #[test]
    fn test_current_dns_state_get_display_ipv4_empty() {
        let state = CurrentDnsState::new();
        assert_eq!(state.get_display(AddressFamily::IPv4), "Automatic");
    }

    #[test]
    fn test_current_dns_state_get_display_ipv4_with_addresses() {
        let state = CurrentDnsState {
            ipv4: vec!["8.8.8.8".to_string(), "8.8.4.4".to_string()],
            ipv6: vec![],
        };
        assert_eq!(state.get_display(AddressFamily::IPv4), "8.8.8.8, 8.8.4.4");
    }

    #[test]
    fn test_current_dns_state_get_display_ipv6_empty() {
        let state = CurrentDnsState::new();
        assert_eq!(state.get_display(AddressFamily::IPv6), "Automatic");
    }

    #[test]
    fn test_current_dns_state_get_display_ipv6_with_addresses() {
        let state = CurrentDnsState {
            ipv4: vec![],
            ipv6: vec!["2001:4860:4860::8888".to_string()],
        };
        assert_eq!(
            state.get_display(AddressFamily::IPv6),
            "2001:4860:4860::8888"
        );
    }

    #[test]
    fn test_window_state_default() {
        let state = WindowState::default();
        assert_eq!(state.x, 100);
        assert_eq!(state.y, 100);
        assert_eq!(state.width, 850);
        assert_eq!(state.height, 700);
        assert!(!state.maximized);
    }

    #[test]
    fn test_window_state_min_constants() {
        assert_eq!(WindowState::MIN_WIDTH, 400);
        assert_eq!(WindowState::MIN_HEIGHT, 300);
    }

    #[test]
    fn test_window_state_serialization() {
        let state = WindowState {
            x: 200,
            y: 150,
            width: 1024,
            height: 768,
            maximized: true,
        };
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: WindowState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, deserialized);
    }

    #[test]
    fn test_app_config_with_window_state() {
        let mut config = AppConfig::new();
        config.window = Some(WindowState {
            x: 300,
            y: 200,
            width: 1280,
            height: 720,
            maximized: false,
        });

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
        assert!(deserialized.window.is_some());
        assert_eq!(deserialized.window.unwrap().width, 1280);
    }

    #[test]
    fn test_app_config_without_window_state() {
        let config = AppConfig::new();
        assert!(config.window.is_none());

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert!(deserialized.window.is_none());
    }
}

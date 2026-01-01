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

#[derive(Clone, PartialEq, Serialize, Deserialize, Default, Debug)]
pub struct AppConfig {
    #[serde(default)]
    pub profiles: Vec<DnsProfile>,
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

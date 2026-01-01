use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, Debug)]
pub enum DnsMode {
    #[default]
    Automatic,
    Manual,
    ManualDoH,
}

impl DnsMode {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            DnsMode::Automatic => "Automatic",
            DnsMode::Manual => "Manual",
            DnsMode::ManualDoH => "Manual (DoH)",
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

#[derive(Clone, Serialize, Deserialize, Debug)]
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

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct DnsEntry {
    pub enabled: bool,
    pub primary: String,
    pub secondary: String,
    pub doh_template: String,
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
        !self.primary.is_empty()
    }

    pub fn get_addresses(&self) -> Vec<String> {
        let mut addresses = Vec::new();
        if !self.primary.is_empty() {
            addresses.push(self.primary.clone());
        }
        if !self.secondary.is_empty() {
            addresses.push(self.secondary.clone());
        }
        addresses
    }
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct DnsSettings {
    pub ipv4: DnsEntry,
    pub ipv6: DnsEntry,
}

impl DnsSettings {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct InterfaceConfig {
    pub interface_guid: String,
    pub interface_name: String,
    pub manual_settings: DnsSettings,
    pub manual_doh_settings: DnsSettings,
}

impl InterfaceConfig {
    pub fn new(interface_guid: String, interface_name: String) -> Self {
        Self {
            interface_guid,
            interface_name,
            manual_settings: DnsSettings::new(),
            manual_doh_settings: DnsSettings::new(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct AppConfig {
    pub interfaces: Vec<InterfaceConfig>,
}

impl AppConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn find_interface(&self, guid: &str) -> Option<&InterfaceConfig> {
        self.interfaces.iter().find(|i| i.interface_guid == guid)
    }

    #[allow(dead_code)]
    pub fn find_interface_mut(&mut self, guid: &str) -> Option<&mut InterfaceConfig> {
        self.interfaces
            .iter_mut()
            .find(|i| i.interface_guid == guid)
    }

    pub fn ensure_interface(&mut self, guid: String, name: String) -> &mut InterfaceConfig {
        if let Some(index) = self
            .interfaces
            .iter()
            .position(|i| i.interface_guid == guid)
        {
            &mut self.interfaces[index]
        } else {
            self.interfaces
                .push(InterfaceConfig::new(guid.clone(), name));
            self.interfaces.last_mut().expect("just pushed an element")
        }
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

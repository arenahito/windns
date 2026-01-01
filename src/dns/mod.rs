pub mod commands;
pub mod config;
pub mod network;
pub mod types;
pub mod validation;

pub use commands::{get_current_dns, set_dns_automatic, set_dns_doh, set_dns_manual};
pub use config::{load_config, save_config};
pub use network::get_network_interfaces;
pub use types::{
    AddressFamily, AppConfig, CurrentDnsState, DnsEntry, DnsMode, DnsSettings, NetworkInterface,
};
pub use validation::{validate_doh_template, validate_ipv4, validate_ipv6};

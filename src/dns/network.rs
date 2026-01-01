use crate::dns::types::NetworkInterface;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("Windows API error: {0}")]
    WindowsApi(String),
    #[error("No network interfaces found")]
    NoInterfaces,
}

pub type Result<T> = std::result::Result<T, NetworkError>;

const AF_INET: u16 = 2;
const AF_INET6: u16 = 23;

#[cfg(target_os = "windows")]
pub fn get_network_interfaces() -> Result<Vec<NetworkInterface>> {
    use windows::Win32::NetworkManagement::IpHelper::{
        GetAdaptersAddresses, GAA_FLAG_SKIP_ANYCAST, GAA_FLAG_SKIP_DNS_SERVER,
        GAA_FLAG_SKIP_MULTICAST, IP_ADAPTER_ADDRESSES_LH,
    };
    use windows::Win32::Networking::WinSock::{AF_UNSPEC, SOCKADDR_IN, SOCKADDR_IN6};

    let mut interfaces = Vec::new();
    let flags = GAA_FLAG_SKIP_ANYCAST | GAA_FLAG_SKIP_MULTICAST | GAA_FLAG_SKIP_DNS_SERVER;

    let mut buffer_size: u32 = 15000;
    let mut buffer: Vec<u8> = vec![0; buffer_size as usize];

    unsafe {
        let result = GetAdaptersAddresses(
            AF_UNSPEC.0 as u32,
            flags,
            None,
            Some(buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH),
            &mut buffer_size,
        );

        if result != 0 {
            return Err(NetworkError::WindowsApi(format!(
                "GetAdaptersAddresses failed with code {}",
                result
            )));
        }

        let mut current = buffer.as_ptr() as *const IP_ADAPTER_ADDRESSES_LH;

        while !current.is_null() {
            let adapter = &*current;

            if adapter.OperStatus.0 == 1 {
                let name = if !adapter.FriendlyName.is_null() {
                    let len = (0..)
                        .take_while(|&i| *adapter.FriendlyName.0.offset(i) != 0)
                        .count();
                    let slice = std::slice::from_raw_parts(adapter.FriendlyName.0, len);
                    String::from_utf16_lossy(slice)
                } else {
                    "Unknown".to_string()
                };

                let guid = if !adapter.AdapterName.is_null() {
                    let c_str = std::ffi::CStr::from_ptr(adapter.AdapterName.0 as *const i8);
                    c_str.to_string_lossy().to_string()
                } else {
                    String::new()
                };

                let mut has_ipv4 = false;
                let mut has_ipv6 = false;

                let mut unicast = adapter.FirstUnicastAddress;
                while !unicast.is_null() {
                    let addr = &*unicast;
                    if !addr.Address.lpSockaddr.is_null() {
                        let sockaddr = &*addr.Address.lpSockaddr;
                        match sockaddr.sa_family.0 {
                            AF_INET => {
                                let ipv4_addr = &*(addr.Address.lpSockaddr as *const SOCKADDR_IN);
                                if ipv4_addr.sin_addr.S_un.S_addr != 0 {
                                    has_ipv4 = true;
                                }
                            }
                            AF_INET6 => {
                                let ipv6_addr = &*(addr.Address.lpSockaddr as *const SOCKADDR_IN6);
                                let is_not_zero =
                                    ipv6_addr.sin6_addr.u.Byte.iter().any(|&b| b != 0);
                                if is_not_zero {
                                    has_ipv6 = true;
                                }
                            }
                            _ => {}
                        }
                    }
                    unicast = addr.Next;
                }

                if has_ipv4 || has_ipv6 {
                    interfaces.push(NetworkInterface {
                        name,
                        interface_index: adapter.Anonymous1.Anonymous.IfIndex,
                        interface_guid: guid,
                        has_ipv4,
                        has_ipv6,
                    });
                }
            }

            current = adapter.Next;
        }
    }

    if interfaces.is_empty() {
        return Err(NetworkError::NoInterfaces);
    }

    Ok(interfaces)
}

#[cfg(not(target_os = "windows"))]
pub fn get_network_interfaces() -> Result<Vec<NetworkInterface>> {
    Err(NetworkError::WindowsApi(
        "Not supported on this platform".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "windows")]
    fn test_get_network_interfaces() {
        let result = get_network_interfaces();
        match result {
            Ok(interfaces) => {
                assert!(!interfaces.is_empty());
                for interface in interfaces {
                    assert!(!interface.name.is_empty());
                    assert!(interface.has_ipv4 || interface.has_ipv6);
                }
            }
            Err(e) => {
                println!("Warning: Could not get network interfaces: {}", e);
            }
        }
    }
}

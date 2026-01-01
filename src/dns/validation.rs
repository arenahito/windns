use std::net::{Ipv4Addr, Ipv6Addr};

pub fn validate_ipv4(addr: &str) -> bool {
    if addr.trim().is_empty() {
        return true;
    }
    addr.parse::<Ipv4Addr>().is_ok()
}

pub fn validate_ipv6(addr: &str) -> bool {
    if addr.trim().is_empty() {
        return true;
    }
    addr.parse::<Ipv6Addr>().is_ok()
}

pub fn validate_doh_template(template: &str) -> bool {
    if template.trim().is_empty() {
        return true;
    }
    template.starts_with("https://") && template.contains("{?dns}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_ipv4() {
        assert!(validate_ipv4(""));
        assert!(validate_ipv4("8.8.8.8"));
        assert!(validate_ipv4("1.1.1.1"));
        assert!(validate_ipv4("192.168.1.1"));
        assert!(!validate_ipv4("256.1.1.1"));
        assert!(!validate_ipv4("invalid"));
        assert!(!validate_ipv4("2001:4860:4860::8888"));
    }

    #[test]
    fn test_validate_ipv6() {
        assert!(validate_ipv6(""));
        assert!(validate_ipv6("2001:4860:4860::8888"));
        assert!(validate_ipv6("2606:4700:4700::1111"));
        assert!(validate_ipv6("::1"));
        assert!(!validate_ipv6("8.8.8.8"));
        assert!(!validate_ipv6("invalid"));
    }

    #[test]
    fn test_validate_doh_template() {
        assert!(validate_doh_template(""));
        assert!(validate_doh_template("https://dns.google/dns-query{?dns}"));
        assert!(validate_doh_template(
            "https://cloudflare-dns.com/dns-query{?dns}"
        ));
        assert!(!validate_doh_template("http://dns.google/dns-query{?dns}"));
        assert!(!validate_doh_template("https://dns.google/dns-query"));
        assert!(!validate_doh_template("invalid"));
    }
}

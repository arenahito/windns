use crate::dns::types::CurrentDnsState;
use thiserror::Error;
use tokio::process::Command;

#[derive(Error, Debug)]
pub enum DnsCommandError {
    #[error("PowerShell command failed: {0}")]
    CommandFailed(String),
    #[error("Registry configuration failed: {0}")]
    RegistryFailed(String),
    #[error("DNS settings applied, but DoH configuration failed: {0}")]
    DnsAppliedButDohFailed(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid output format")]
    InvalidOutput,
}

pub type Result<T> = std::result::Result<T, DnsCommandError>;

const AF_INET: u64 = 2;
const AF_INET6: u64 = 23;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

fn escape_powershell_string(s: &str) -> String {
    s.replace('`', "``")
        .replace("'", "''")
        .replace(['\n', '\r'], "")
}

fn normalize_guid(guid: &str) -> String {
    guid.trim_matches(['{', '}'].as_ref()).to_string()
}

fn normalize_error_message(msg: &str) -> String {
    msg.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

async fn run_powershell(script: &str) -> Result<String> {
    let script_with_setup = format!(
        "[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; $ErrorActionPreference = 'Stop'; {}",
        script
    );
    let mut command = Command::new("powershell.exe");
    command.args([
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        &script_with_setup,
    ]);

    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);

    let output = command.output().await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(DnsCommandError::CommandFailed(normalize_error_message(
            &stderr,
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub async fn get_current_dns(interface_index: u32) -> Result<CurrentDnsState> {
    let script = format!(
        "Get-DnsClientServerAddress -InterfaceIndex {} | ConvertTo-Json -Compress",
        interface_index
    );

    let output = run_powershell(&script).await?;

    let mut state = CurrentDnsState::new();

    if output.trim().is_empty() || output.trim() == "null" {
        return Ok(state);
    }

    let json_value: serde_json::Value =
        serde_json::from_str(&output).map_err(|_| DnsCommandError::InvalidOutput)?;

    let entries = if json_value.is_array() {
        json_value.as_array().expect("checked is_array").clone()
    } else {
        vec![json_value]
    };

    for entry in entries {
        if let Some(family) = entry.get("AddressFamily").and_then(|v| v.as_u64())
            && let Some(addresses) = entry.get("ServerAddresses").and_then(|v| v.as_array())
        {
            let addr_list: Vec<String> = addresses
                .iter()
                .filter_map(|a| a.as_str().map(|s| s.to_string()))
                .collect();

            match family {
                AF_INET => state.ipv4 = addr_list,
                AF_INET6 => state.ipv6 = addr_list,
                _ => {}
            }
        }
    }

    Ok(state)
}

pub async fn set_dns_automatic(interface_index: u32) -> Result<()> {
    let script = format!(
        "Set-DnsClientServerAddress -InterfaceIndex {} -ResetServerAddresses",
        interface_index
    );

    run_powershell(&script).await?;

    Ok(())
}

pub async fn set_dns_manual(interface_index: u32, addresses: Vec<String>) -> Result<()> {
    if addresses.is_empty() {
        return set_dns_automatic(interface_index).await;
    }

    let addr_list = addresses
        .iter()
        .map(|a| format!("'{}'", escape_powershell_string(a)))
        .collect::<Vec<_>>()
        .join(",");

    let script = format!(
        "Set-DnsClientServerAddress -InterfaceIndex {} -ServerAddresses @({})",
        interface_index, addr_list
    );

    run_powershell(&script).await?;

    Ok(())
}

async fn configure_doh_for_server(
    address: &str,
    template: &str,
    allow_fallback: bool,
) -> Result<()> {
    let fallback_str = if allow_fallback { "$true" } else { "$false" };
    let escaped_address = escape_powershell_string(address);
    let escaped_template = escape_powershell_string(template);

    let script = format!(
        r#"
        $addr = '{}'
        $existing = Get-DnsClientDohServerAddress -ServerAddress $addr -ErrorAction SilentlyContinue
        if ($existing) {{
            Set-DnsClientDohServerAddress -ServerAddress $addr -DohTemplate '{}' -AllowFallbackToUdp {} -AutoUpgrade $true
        }} else {{
            Add-DnsClientDohServerAddress -ServerAddress $addr -DohTemplate '{}' -AllowFallbackToUdp {} -AutoUpgrade $true
        }}
        "#,
        escaped_address, escaped_template, fallback_str, escaped_template, fallback_str
    );

    run_powershell(&script).await?;
    Ok(())
}

async fn enable_doh_registry(interface_guid: &str) -> Result<()> {
    let normalized_guid = normalize_guid(interface_guid);
    let escaped_guid = escape_powershell_string(&normalized_guid);
    let script = format!(
        r#"
        $regPath = 'HKLM:\SYSTEM\CurrentControlSet\Services\Dnscache\InterfaceSpecificParameters\{{{}}}'
        if (-not (Test-Path $regPath)) {{
            New-Item -Path $regPath -Force | Out-Null
        }}
        $propName = 'DohFlags'
        $existingProp = Get-ItemProperty -Path $regPath -Name $propName -ErrorAction SilentlyContinue
        if ($existingProp) {{
            Set-ItemProperty -Path $regPath -Name $propName -Value 1 -Force
        }} else {{
            New-ItemProperty -Path $regPath -Name $propName -Value 1 -PropertyType DWord -Force | Out-Null
        }}
        "#,
        escaped_guid
    );

    run_powershell(&script).await.map_err(|e| {
        DnsCommandError::RegistryFailed(match e {
            DnsCommandError::CommandFailed(msg) => msg,
            other => other.to_string(),
        })
    })?;
    Ok(())
}

/// Attempts to configure DoH for a server.
/// Returns (was_attempted: bool, error: Option<String>)
/// - (false, None): DoH not applicable (not enabled or empty config)
/// - (true, None): DoH configured successfully
/// - (true, Some(err)): DoH configuration failed
async fn try_configure_doh(
    server: &crate::dns::DnsServerEntry,
    label: &str,
) -> (bool, Option<String>) {
    if server.doh_mode != crate::dns::DohMode::On
        || server.address.is_empty()
        || server.doh_template.is_empty()
    {
        return (false, None);
    }

    match configure_doh_for_server(&server.address, &server.doh_template, server.allow_fallback)
        .await
    {
        Ok(()) => (true, None),
        Err(e) => (
            true,
            Some(format!(
                "{}: {}",
                label,
                normalize_error_message(&e.to_string())
            )),
        ),
    }
}

/// Result type for DNS settings application
/// - Ok(None): Complete success
/// - Ok(Some(warning)): DNS applied, some DoH configs failed but at least one succeeded
/// - Err(DnsAppliedButDohFailed): DNS applied, but all DoH configs failed or registry failed
/// - Err(other): DNS application itself failed
pub async fn set_dns_with_settings(
    interface_index: u32,
    interface_guid: &str,
    settings: &crate::dns::DnsSettings,
) -> Result<Option<String>> {
    let mut all_addresses: Vec<String> = Vec::new();

    if settings.ipv4.enabled {
        all_addresses.extend(settings.ipv4.get_addresses());
    }
    if settings.ipv6.enabled {
        all_addresses.extend(settings.ipv6.get_addresses());
    }

    let mut seen = std::collections::HashSet::new();
    all_addresses.retain(|addr| seen.insert(addr.clone()));

    if all_addresses.is_empty() {
        set_dns_automatic(interface_index).await?;
        return Ok(None);
    }

    set_dns_manual(interface_index, all_addresses).await?;

    let mut doh_errors: Vec<String> = Vec::new();
    let mut any_doh_succeeded = false;
    let mut any_doh_attempted = false;

    if settings.ipv4.enabled {
        let (was_attempted, error) =
            try_configure_doh(&settings.ipv4.primary, "IPv4 Primary").await;
        if was_attempted {
            any_doh_attempted = true;
            if let Some(e) = error {
                doh_errors.push(e);
            } else {
                any_doh_succeeded = true;
            }
        }

        let (was_attempted, error) =
            try_configure_doh(&settings.ipv4.secondary, "IPv4 Secondary").await;
        if was_attempted {
            any_doh_attempted = true;
            if let Some(e) = error {
                doh_errors.push(e);
            } else {
                any_doh_succeeded = true;
            }
        }
    }

    if settings.ipv6.enabled {
        let (was_attempted, error) =
            try_configure_doh(&settings.ipv6.primary, "IPv6 Primary").await;
        if was_attempted {
            any_doh_attempted = true;
            if let Some(e) = error {
                doh_errors.push(e);
            } else {
                any_doh_succeeded = true;
            }
        }

        let (was_attempted, error) =
            try_configure_doh(&settings.ipv6.secondary, "IPv6 Secondary").await;
        if was_attempted {
            any_doh_attempted = true;
            if let Some(e) = error {
                doh_errors.push(e);
            } else {
                any_doh_succeeded = true;
            }
        }
    }

    if any_doh_attempted && !any_doh_succeeded {
        return Err(DnsCommandError::DnsAppliedButDohFailed(
            doh_errors.join("; "),
        ));
    }

    if any_doh_succeeded {
        enable_doh_registry(interface_guid).await.map_err(|e| {
            DnsCommandError::DnsAppliedButDohFailed(format!(
                "Registry configuration failed: {}",
                normalize_error_message(&match e {
                    DnsCommandError::RegistryFailed(msg) => msg,
                    other => other.to_string(),
                })
            ))
        })?;
    }

    if !doh_errors.is_empty() {
        return Ok(Some(format!(
            "Some DoH configurations failed: {}",
            doh_errors.join("; ")
        )));
    }

    Ok(None)
}

pub async fn clear_dns_cache() -> Result<()> {
    let script = "Clear-DnsClientCache";
    run_powershell(script).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_clear_dns_cache() {
        let result = clear_dns_cache().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn test_powershell_execution() {
        let result = run_powershell("Write-Output 'test'").await;
        assert!(result.expect("should succeed").contains("test"));
    }

    #[test]
    fn test_escape_powershell_string() {
        assert_eq!(escape_powershell_string("test"), "test");
        assert_eq!(escape_powershell_string("it's"), "it''s");
        assert_eq!(escape_powershell_string("back`tick"), "back``tick");
        assert_eq!(escape_powershell_string("new\nline"), "newline");
    }

    #[test]
    fn test_normalize_guid() {
        assert_eq!(normalize_guid("{ABC-123}"), "ABC-123");
        assert_eq!(normalize_guid("ABC-123"), "ABC-123");
        assert_eq!(normalize_guid("{}"), "");
    }
}

use crate::dns::types::{AddressFamily, CurrentDnsState};
use thiserror::Error;
use tokio::process::Command;

#[derive(Error, Debug)]
pub enum DnsCommandError {
    #[error("PowerShell command failed: {0}")]
    CommandFailed(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid output format")]
    InvalidOutput,
}

pub type Result<T> = std::result::Result<T, DnsCommandError>;

const AF_INET: u64 = 2;
const AF_INET6: u64 = 23;

fn escape_powershell_string(s: &str) -> String {
    s.replace("'", "''")
}

async fn run_powershell(script: &str) -> Result<String> {
    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(DnsCommandError::CommandFailed(stderr.to_string()));
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

pub async fn set_dns_automatic(interface_index: u32, family: AddressFamily) -> Result<()> {
    let family_str = match family {
        AddressFamily::IPv4 => "IPv4",
        AddressFamily::IPv6 => "IPv6",
    };

    let script = format!(
        "Set-DnsClientServerAddress -InterfaceIndex {} -ResetServerAddresses -AddressFamily {}",
        interface_index, family_str
    );

    run_powershell(&script).await?;
    clear_dns_cache().await?;

    Ok(())
}

pub async fn set_dns_manual(
    interface_index: u32,
    family: AddressFamily,
    addresses: Vec<String>,
) -> Result<()> {
    if addresses.is_empty() {
        return set_dns_automatic(interface_index, family).await;
    }

    let family_str = match family {
        AddressFamily::IPv4 => "IPv4",
        AddressFamily::IPv6 => "IPv6",
    };

    let addr_list = addresses
        .iter()
        .map(|a| format!("'{}'", escape_powershell_string(a)))
        .collect::<Vec<_>>()
        .join(",");

    let script = format!(
        "Set-DnsClientServerAddress -InterfaceIndex {} -ServerAddresses @({}) -AddressFamily {}",
        interface_index, addr_list, family_str
    );

    run_powershell(&script).await?;
    clear_dns_cache().await?;

    Ok(())
}

pub async fn set_dns_doh(
    interface_index: u32,
    family: AddressFamily,
    addresses: Vec<String>,
    doh_template: String,
) -> Result<()> {
    set_dns_manual(interface_index, family, addresses.clone()).await?;

    if !doh_template.is_empty() && !addresses.is_empty() {
        for address in addresses {
            let script = format!(
                "Add-DnsClientDohServerAddress -ServerAddress '{}' -DohTemplate '{}' -AllowFallbackToUdp $true -AutoUpgrade $true",
                escape_powershell_string(&address),
                escape_powershell_string(&doh_template)
            );

            if let Err(e) = run_powershell(&script).await {
                eprintln!("Warning: Failed to register DoH server {}: {}", address, e);
            }
        }

        enable_doh_registry(interface_index).await;
    }

    clear_dns_cache().await?;

    Ok(())
}

async fn enable_doh_registry(interface_index: u32) {
    let script = format!(
        r#"
        $regPath = "HKLM:\SYSTEM\CurrentControlSet\Services\Dnscache\InterfaceSpecificParameters\{}"
        if (-not (Test-Path $regPath)) {{
            New-Item -Path $regPath -Force | Out-Null
        }}
        Set-ItemProperty -Path $regPath -Name "DohFlags" -Value 1 -Type DWord -Force
        "#,
        interface_index
    );

    if let Err(e) = run_powershell(&script).await {
        eprintln!(
            "Warning: Failed to enable DoH registry for interface {}: {}",
            interface_index, e
        );
    }
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
    async fn test_clear_dns_cache() {
        let result = clear_dns_cache().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_powershell_execution() {
        let result = run_powershell("Write-Output 'test'").await;
        assert!(result.expect("should succeed").contains("test"));
    }
}

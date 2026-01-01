# Windows DNS Switcher

A GUI tool to switch Windows 11 DNS settings between Automatic/Manual/Manual(DoH) modes. Supports IPv4/IPv6 and saves manual settings per network interface to JSONC file.

## Features

- **Multiple DNS Modes**
  - Automatic (DHCP)
  - Manual
  - Manual with DNS over HTTPS (DoH)

- **IPv4 and IPv6 Support**
  - Separate configuration for IPv4 and IPv6
  - Enable/disable each protocol independently

- **Network Interface Management**
  - Automatically detects active network interfaces
  - Switch between different network adapters

- **Settings Persistence**
  - Saves manual DNS settings per network interface
  - Configuration stored in JSONC format at `%APPDATA%\windns\config.jsonc`

- **Modern UI**
  - Dark theme
  - Clean and intuitive interface
  - Real-time DNS status display

## Requirements

- Windows 11 (or Windows 10 with PowerShell 5.1+)
- Administrator privileges (required for DNS changes)
- Rust 1.92.0 or later (for building from source)

## Building

```bash
cargo build --release
```

The executable will be located at `target/release/windns.exe`.

## Usage

1. Run the application as Administrator
2. Select your network interface from the dropdown
3. Choose DNS mode:
   - **Automatic**: Use DHCP-provided DNS servers
   - **Manual**: Set custom DNS servers
   - **Manual (DoH)**: Set custom DNS servers with DNS over HTTPS support
4. Configure IPv4 and/or IPv6 DNS settings
5. Click "Apply" to save and apply settings
6. Click "Reset" to revert to saved settings

## DNS Configuration

### Manual Mode

1. Enable IPv4 or IPv6 (or both)
2. Enter primary DNS server address (required)
3. Enter secondary DNS server address (optional)
4. Click "Apply"

### Manual (DoH) Mode

1. Enable IPv4 or IPv6 (or both)
2. Enter primary DNS server address (required)
3. Enter secondary DNS server address (optional)
4. Enter DoH template URL (optional)
   - Example: `https://dns.google/dns-query{?dns}`
   - Must start with `https://` and contain `{?dns}`
5. Click "Apply"

## Popular DNS Servers

### IPv4

- **Google DNS**: 8.8.8.8, 8.8.4.4
- **Cloudflare DNS**: 1.1.1.1, 1.0.0.1
- **Quad9 DNS**: 9.9.9.9, 149.112.112.112

### IPv6

- **Google DNS**: 2001:4860:4860::8888, 2001:4860:4860::8844
- **Cloudflare DNS**: 2606:4700:4700::1111, 2606:4700:4700::1001
- **Quad9 DNS**: 2620:fe::fe, 2620:fe::9

### DoH Templates

- **Google**: `https://dns.google/dns-query{?dns}`
- **Cloudflare**: `https://cloudflare-dns.com/dns-query{?dns}`
- **Quad9**: `https://dns.quad9.net/dns-query{?dns}`

## Technology Stack

- **Rust 1.92.0**
- **Dioxus 0.5.x** - Desktop GUI framework
- **dioxus-free-icons** - Icon library
- **PowerShell** - DNS operations
- **Windows API** - Network interface detection
- **JSONC** - Configuration file format

## Project Structure

```
windns/
├── Cargo.toml
├── Dioxus.toml
├── build.rs
├── windns.manifest
├── assets/
│   └── main.css
└── src/
    ├── main.rs
    ├── app.rs
    ├── state.rs
    ├── components/
    │   ├── mod.rs
    │   ├── header.rs
    │   ├── network_selector.rs
    │   ├── dns_mode_selector.rs
    │   ├── dns_tabs.rs
    │   ├── dns_input.rs
    │   ├── status_bar.rs
    │   └── action_buttons.rs
    └── dns/
        ├── mod.rs
        ├── types.rs
        ├── config.rs
        ├── network.rs
        ├── commands.rs
        └── validation.rs
```

## License

This project is provided as-is without any warranty.

## Notes

- Administrator privileges are required because DNS settings modification requires elevated permissions
- DNS cache is automatically cleared after every settings change
- Settings are saved automatically when switching between manual modes
- The application only shows active network interfaces with IPv4 or IPv6 addresses

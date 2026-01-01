# Windows DNS Switcher

A GUI tool to switch Windows 11 DNS settings between Automatic/Manual/Manual(DoH) modes. Supports IPv4/IPv6 and saves manual settings per network interface to JSONC file.

## Features

- **Multiple DNS Modes**
  - Automatic (DHCP)
  - Manual (with optional DoH support)

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

## Usage

1. Run the application as Administrator
2. Select your network interface from the dropdown
3. Choose DNS mode:
   - **Automatic**: Use DHCP-provided DNS servers
   - **Manual**: Set custom DNS servers (with optional DoH support)
4. Configure IPv4 and/or IPv6 DNS settings
5. Click "Apply" to save and apply settings
6. Click "Reset" to revert to saved settings

## DNS Configuration

### Manual Mode

1. Enable IPv4 or IPv6 (or both)
2. Enter primary DNS server address (required)
3. Enter secondary DNS server address (optional)
4. Configure DoH settings (optional):
   - Enable DoH toggle
   - Enter DoH template URL
   - Example: `https://dns.google/dns-query`
   - Must start with `https://`
5. Click "Apply"

## License

This project is provided as-is without any warranty.

## Notes

- Administrator privileges are required because DNS settings modification requires elevated permissions
- DNS cache is automatically cleared after every settings change
- Settings are saved automatically when switching between manual modes
- The application only shows active network interfaces with IPv4 or IPv6 addresses

# Hope OS — Installation Guide

> *"9 minutes, 0 jargon."*

## Quick Install

```bash
# Download the ISO
wget https://hope-os.org/releases/hope-os-0.1.0-alpha.iso

# Create bootable USB
sudo dd if=hope-os-0.1.0-alpha.iso of=/dev/sdX bs=4M status=progress

# Boot from USB and follow the 8 screens
```

## Installation Screens

### 1. Welcome
Choose your language.

### 2. Disk
- **Simple**: Use entire disk with recommended settings
- **Advanced**: Manual partitioning, LUKS2 encryption, Btrfs

### 3. User
Create your account:
- Full name
- Username
- Password (with strength indicator)

### 4. Vault
Choose your password manager:
- **Bitwarden** — Recommended, open source
- **KeePassXC** — Local, full control
- **vaultwarden** — Self-hosted
- **Proton Pass** — If you use Proton Mail/VPN
- **Plus tard** — Skip for now

### 5. Privacy Level
Choose your threat model:

| Level | Protection | Performance |
|---|---|---|
| **Standard** | DoH Cloudflare, Hope-Mind local | Best |
| **Renforcé** | DNS over Tor, vault unlock-once | Good |
| **Maximum** | Tor proxy, LUKS2 mandatory, RAM wipe | Good |

### 6. Hope-Mind
Enable AI features:
- **Activer** — Full AI integration
- **Désactiver** — Privacy-focused, no AI

### 7. Import
Optionally import from Windows:
- Bookmarks
- Documents
- Wallpaper

### 8. Ready
Review settings and install.

## System Requirements

| Requirement | Minimum | Recommended |
|---|---|---|
| RAM | 2GB | 8GB+ |
| Storage | 20GB | 50GB+ |
| CPU | x86_64 | x86_64 with AES-NI |
| Boot | UEFI | UEFI |

## Post-Install

### First Boot
1. Remove USB drive
2. Power on
3. Snapshots are created automatically

### Daily Commands

```bash
# Check system status
hope-ctl status

# List snapshots
btrfs snapshot list /

# Rollback to last snapshot
btrfs rollback

# Update Hope OS
hope-ctl update

# View HAL+ logs
journalctl -u hope-hal -f
```

## Troubleshooting

### Boot Issues
1. Check BIOS boot order (UEFI first)
2. Disable Secure Boot if issues persist
3. Try "Safe Mode" from boot menu

### Hardware Not Working
1. Open terminal
2. Run `hope-hal status`
3. Check for missing drivers
4. Report via `hope-ctl report`

### Performance
- Boot time > 4s: Check `systemd-analyze`
- High RAM: `free -h`
- Check for snapshot accumulation: `btrfs snapshot list /`

## Uninstall

Boot from USB → Advanced → Format disk

## Support

- Docs: https://hope-os.org/docs
- Issues: https://github.com/lvs0/Hope/issues
- Matrix: #hope-os:matrix.org

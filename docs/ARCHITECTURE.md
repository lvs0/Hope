# Hope OS — Architecture

> *"Your digital life, under your control."*

## System Overview

Hope OS is a Linux-based operating system designed around user privacy and control.

```
┌─────────────────────────────────────────────────────────────┐
│                    Hope Shell (Wayland)                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │  Spotlight  │  │ Hope AI Panel │  │  Hope Voice       │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                     Hope Mind (AI)                           │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │  Spotlight  │  │  Deep Work   │  │      Whisper        │ │
│  │ Phi-3.5-mini│  │ Granite 3.1 │  │   (voice local)     │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                   Hope OS Services                           │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │  Hope Halo  │  │ Hope Vault  │  │    Polygone         │ │
│  │ (HAL+ Rust) │  │ (security)  │  │  (network sync)     │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                   Debian Stable 12                          │
│                   linux-zen kernel                          │
└─────────────────────────────────────────────────────────────┘
```

## Components

### HAL+ — Hardware Adaptation Layer

Rust daemon that handles hardware detection and driver management.

**Flow:**
```
udev event → HAL+ receives vendor:product ID
→ Lookup in SQLite database
→ If found: install → configure → notification
→ If unknown: request cloud lookup
```

**Key files:**
- `hal/src/main.rs` — Main daemon loop
- `hal/src/db.rs` — SQLite driver database
- `hal/src/detection.rs` — udev monitoring
- `hal/src/drivers.rs` — Driver installation
- `hal/src/notifications.rs` — User notifications

### Hope Shell — Wayland Compositor

Wayland compositor based on wlroots with Deep Space theme.

**Theme colors:**
- Background: `#0F0F12`
- Accents: indigo `#4F46E5`, violet `#7C3AED`, cyan `#06B6D4`
- Font: DM Mono

### Hope Vault — Security System

Manages password and credential security:
- **Bitwarden** (recommended)
- **KeePassXC** (local-first)
- **vaultwarden** (self-hosted)
- **Proton Pass** (Proton ecosystem)

**Encryption:**
- LUKS2 full-disk (mandatory for Maximum mode)
- Argon2id KDF
- RAM wiped on shutdown (Maximum mode)
- Clipboard auto-clear after 2min

### Hope Mind — AI Integration

Local AI models for system intelligence:

| Model | Purpose | Size |
|---|---|---|
| Phi-3.5-mini Q2_K | Spotlight launcher | < 2GB |
| Granite 3.1-2B | Deep work tasks | ~4GB |
| Whisper-tiny | Voice transcription | 75MB |

**Smart loading:**
- Idle > 15min → unload all models
- Super pressed → load Phi from mmap
- Deep Work → Granite to RAM
- Voice → Whisper on demand

### Polygone Integration

Each Hope OS installation is a Polygone node:

- **Hope Sync** — E2E encrypted file sync
- **Compute sharing** — Distributed LLM inference
- **Hope Cast** — Screen sharing < 50ms
- **Dev sharing** — Port forwarding without ngrok

## Security Levels

| Level | Encryption | RAM wipe | DNS |
|---|---|---|---|
| Standard | Bitwarden | No | DoH Cloudflare |
| Renforcé | LUKS2 | No | DoH over Tor |
| Maximum | LUKS2 | Yes | Tor proxy |

## Boot Targets

| Target | Description |
|---|---|
| `hope-os` | Standard boot |
| `hope-os (Safe Mode)` | Safe graphics mode |
| `hope-os (Btrfs Snapshot)` | Rollback snapshot |

## File System Layout

```
/
├── @              # Btrfs subvolume (root)
├── @home          # Btrfs subvolume (home)
├── @var           # Btrfs subvolume (var)
├── @snapshots     # Btrfs subvolume (snapshots)
└── boot/efi       # EFI partition
```

## Technical Stack

| Component | Technology |
|---|---|
| Base OS | Debian Stable 12 |
| Kernel | linux-zen + localmodconfig |
| Shell | wlroots (Wayland) |
| HAL+ | Rust |
| AI models | Phi-3.5-mini, Granite 3.1, Whisper |
| Cryptography | ML-KEM-1024, ML-DSA-87, BLAKE3 |
| Filesystem | Polygone-Drive |
| Vault | Bitwarden / KeePassXC |

# Hope OS — Specification

## Overview

**Hope OS** is a Debian-based Linux distribution designed for beginners who want an OS that just works. Built on a Debian kernel with Rust user-facing components.

**Target:** Beginners who want results without friction.

---

## Architecture

```
hope-iso/
└── hope/                  # SquashFS root
    ├── etc/               # System config
    ├── usr/               # Apps + libs
    └── home/              # Default user dir
```

### Directory Structure

- `etc/` — System configuration files (hostname, passwd, debian_version, etc.)
- `usr/` — User-space binaries and libraries
- `home/hope/` — Default user directory

---

## System Configuration

- **Hostname:** `hope`
- **Default User:** `hope` / `hope`
- **Base:** Debian kernel

### Files in `/etc/`

| File | Purpose |
|------|---------|
| `debian_version` | Debian base version |
| `hostname` | System hostname (`hope`) |
| `passwd` | User accounts |
| `shadow` | Password hashes |

---

## Build System

### Requirements

- `debootstrap`
- `xorriso`
- `squashfs-tools`

### Build ISO

```bash
./scripts/build-iso.sh
```

### Install Dependencies

```bash
./scripts/setup-deps.sh
```

---

## Components (Planned)

- [ ] Rust package manager integration
- [ ] Auto-update system
- [ ] Opinionated defaults for beginners
- [ ] Zero-config networking
- [ ] Desktop environment (TBD)

---

## Version

**v0.1** — Foundation

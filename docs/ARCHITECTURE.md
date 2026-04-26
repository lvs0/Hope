# Hope OS Architecture

_Under construction — v0.1_

## Design Goals

1. **Works out of the box** — No configuration required to get a working system
2. **Beginner-friendly** — Clear defaults, no obscure Linux knowledge needed
3. **Debian foundation** — Stable, proven base with access to the Debian ecosystem
4. **Rust-powered** — Modern, safe user-space tools

## System Overview

```
+------------------+
|    Hope OS      |
|  (User Space)   |  <- Rust applications & tools
+------------------+
|   Debian Base   |  <- Kernel + coreutils
+------------------+
|   Bootloader    |
+------------------+
```

## Core Components

### 1. Kernel Layer
- **Base:** Debian Linux kernel
- **Why Debian:** Stability, massive package repository, proven enterprise-grade

### 2. System Files (`/etc/`)
- `hostname` — System identification
- `passwd` — User database
- `debian_version` — Base version tracking

### 3. User Space (`/usr/`)
- **Binaries:** Core executables
- **Libraries:** Shared libraries for Rust components

### 4. Home Directory (`/home/hope/`)
- Pre-configured user directory for default `hope` user
- Sensible defaults for new users

## Build Process

```
debootstrap → squashfs → ISO9660 (xorriso)
```

1. **debootstrap** — Pulls Debian base into `hope-iso/hope/`
2. **squashfs** — Compress root filesystem
3. **xorriso** — Wrap into bootable ISO

## Future Components

- [ ] `hope-spawn` — Application launcher (see `/usr/bin/hope-spawn`)
- [ ] Auto-update via Rust updater
- [ ] Zero-config networking manager
- [ ] Opinionated desktop environment

---

_Built with 🧠 by the Polygone ecosystem_

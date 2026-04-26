# Hope OS

**Linux for humans who want results.**

Hope OS is a beginner-friendly distribution built on the Debian kernel with Rust-powered user-space tools.

---

## Quick Start

```bash
# Install dependencies
./scripts/setup-deps.sh

# Build the ISO
./scripts/build-iso.sh
```

The resulting ISO will be at `hope-iso/hope-os.iso`.

---

## Requirements

- `debootstrap` — Bootstrap Debian base system
- `xorriso` — ISO image creation
- `squashfs-tools` — SquashFS filesystem tools

On Debian/Ubuntu:
```bash
sudo apt install debootstrap xorriso squashfs-tools
```

---

## Default Login

```
Username:  hope
Password:  hope
```

---

## Project Structure

```
Hope/
├── SPEC.md              # Full specifications
├── README.md            # This file
├── hope-iso/            # ISO build contents
│   └── hope/
│       ├── etc/         # System config
│       ├── usr/         # Apps + libs
│       └── home/        # Default user
├── scripts/
│   ├── build-iso.sh    # Build the ISO
│   └── setup-deps.sh   # Install deps
└── docs/
    └── ARCHITECTURE.md  # Architecture details
```

---

## License

Apache 2.0 / MIT (see POLYGONE ecosystem license)

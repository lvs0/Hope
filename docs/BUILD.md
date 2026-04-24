# Building Hope OS

## Prerequisites

```bash
# Debian/Ubuntu
sudo apt install -y \
  build-essential git cargo rustc \
  debootstrap qemu-utils xorriso squashfs-tools \
  grub-pc-bin grub-efi-amd64 \
  libssl-dev pkg-config
```

## Quick Build

```bash
cd build
make          # Full build: HAL+ → kernel → ISO
make hal      # Build only HAL+
make iso      # Build only ISO
```

## Manual Build

### 1. Build HAL+
```bash
cd hal
cargo build --release
sudo cp target/release/hal-daemon /usr/local/bin/
```

### 2. Build Kernel
```bash
cd build/scripts
chmod +x build-kernel.sh
sudo ./build-kernel.sh
```

### 3. Create ISO
```bash
chmod +x make-iso.sh
sudo ./make-iso.sh
```

## Output

- ISO: `output/hope-os.iso`
- Kernel DEBs: `output/*.deb`
- HAL+ binary: `hal/target/release/hal-daemon`

## System Requirements

| | Minimum | Recommended |
|---|---|---|
| RAM | 2GB | 8GB |
| Storage | 20GB | 50GB |
| CPU | 64-bit | Modern multi-core |
| Network | Ethernet | WiFi |

## Build Time

| Step | Time |
|---|---|
| HAL+ | ~2 min |
| Kernel | ~30-60 min (depends on CPU) |
| ISO | ~10 min |

## Troubleshooting

### Kernel won't compile
```bash
# Ensure you have enough RAM
make -j$(nproc)  # Full parallel (needs 8GB+ RAM)
make -j2         # Conservative (2GB RAM)
```

### ISO won't boot
- Verify Secure Boot is disabled in BIOS
- Try with QEMU: `qemu-system-x86_64 -m 4G -cdrom output/hope-os.iso`
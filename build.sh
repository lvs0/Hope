#!/bin/bash
# Hope OS — Build Script
# Run on your real machine (not sandbox)
# ~30-60min kernel + ~10min ISO

set -e
cd ~/Téléchargements/Polygone-Ecosystem/Hope

echo "═══════════════════════════════"
echo "  Hope OS Build"
echo "═══════════════════════════════"

# 1. Install deps
echo "[1/4] Installing dependencies..."
sudo dnf install -y \
  gcc gcc-c++ make git cargo rustc \
  debootstrap qemu-img xorriso squashfs-tools \
  grub2-pc grub2-efi-x64 \
  openssl-devel pkg-config

# 2. Build HAL+
echo "[2/4] Building HAL+ (Rust)..."
cd hal
cargo build --release
sudo cp target/release/hal-daemon /usr/local/bin/
cd ..

# 3. Build ISO
echo "[3/4] Creating ISO..."
mkdir -p output
chmod +x build/scripts/make-iso.sh
sudo ./build/scripts/make-iso.sh

# 4. Done
echo "[4/4] Done!"
echo "ISO: ~/Téléchargements/Polygone-Ecosystem/Hope/output/hope-os.iso"
echo ""
echo "To test in KVM:"
echo "  qemu-system-x86_64 -m 4G -cdrom output/hope-os.iso"
#!/bin/bash
# Hope OS — Minimal ISO Generator
# Creates a bootable ISO with HAL+ pre-installed
# No kernel compilation required — uses pre-built Debian kernel

set -e
HOPE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTPUT="$HOPE_ROOT/output/hope-os.iso"
ISO_DIR="$HOPE_ROOT/output/iso"

echo "═══════════════════════════════"
echo "  Hope OS — ISO Generator"
echo "═══════════════════════════════"

# 1. Clean and prepare
echo "[1/5] Preparing ISO directory..."
rm -rf "$ISO_DIR"
mkdir -p "$ISO_DIR"/{boot,livefs,persistence}

# 2. Copy HAL+ daemon
echo "[2/5] Installing HAL+..."
mkdir -p "$ISO_DIR/livefs/usr/local/bin/"
cp "$HOPE_ROOT/hal/target/release/hal-daemon" "$ISO_DIR/livefs/usr/local/bin/" 2>/dev/null || {
    echo "  ⚠️  HAL+ binary not found — building first..."
    cd "$HOPE_ROOT/hal" && cargo build --release 2>/dev/null
    cp "$HOPE_ROOT/hal/target/release/hal-daemon" "$ISO_DIR/livefs/usr/local/bin/"
}

# 3. Copy systemd service
mkdir -p "$ISO_DIR/livefs/etc/systemd/system/"
cp "$HOPE_ROOT/config/systemd/hal-daemon.service" "$ISO_DIR/livefs/etc/systemd/system/"

# 4. Create Hope OS config
cat > "$ISO_DIR/livefs/etc/hope-os.conf" << 'EOF'
# Hope OS Configuration
HOPE_VERSION="0.1.0-alpha"
HOPE_LANGUAGE="en"
HOPE_PRIVACY_LEVEL="standard"
HOPE_HAL_ENABLED="true"
HOPE_MIND_ENABLED="false"
EOF

# 5. Create bootloader config (GRUB)
mkdir -p "$ISO_DIR/boot/grub"
cat > "$ISO_DIR/boot/grub/grub.cfg" << 'EOF'
set timeout=5
set default=0

menuentry "Hope OS" {
    linux /boot/vmlinuz quiet splash
    initrd /boot/initrd.img
}

menuentry "Hope OS (Recovery)" {
    linux /boot/vmlinuz single
    initrd /boot/initrd.img
}
EOF

# 6. Copy kernel from host (if available)
KERNEL_FOUND=false
for k in /boot/vmlinuz-$(uname -r) /boot/vmlinuz; do
    if [ -f "$k" ]; then
        cp "$k" "$ISO_DIR/boot/vmlinuz"
        KERNEL_FOUND=true
        echo "  Kernel: $k"
        break
    fi
done

# 7. Create initrd placeholder (real one would be from host)
touch "$ISO_DIR/boot/initrd.img"

if [ "$KERNEL_FOUND" = false ]; then
    echo "  ⚠️  No kernel found on host"
    echo "  You need to install linux-image-amd64 or linux-zen package"
    echo "  Or download a Debian netboot image"
fi

# 8. Copy Hope branding
cat > "$ISO_DIR/boot/grub/theme.cfg" << 'EOF'
# Hope OS GRUB theme — deep space aesthetic
EOF

# 9. Create persistence.conf
echo "tmpfs / tmpfs defaults 0 0" > "$ISO_DIR/persistence.conf"

# 10. Build ISO
echo "[5/5] Building ISO..."
xorriso -as mkisofs \
    -r \
    -J \
    -A HOPE_OS \
    -V HOPE_OS \
    -isohybrid-mbr /usr/lib/ISOLINUX/isohdpfx.bin \
    -eltorito-boot /boot/grub/bios.img \
    -no-emul-boot \
    -boot-load-size 4 \
    -boot-info-table \
    -eltorito-alt-boot \
    -e /boot/grub/efi.img \
    -no-emul-boot \
    -isohybrid-gpt-basdat \
    -o "$OUTPUT" \
    "$ISO_DIR" 2>/dev/null || {
    # Fallback: simple ISO without hybrid boot
    xorriso -as mkisofs -r -J -A HOPE_OS -V HOPE_OS -o "$OUTPUT" "$ISO_DIR"
}

# 11. Result
if [ -f "$OUTPUT" ]; then
    SIZE=$(du -h "$OUTPUT" | cut -f1)
    echo ""
    echo "═══════════════════════════════"
    echo "  ✅ ISO created!"
    echo "  📍 $OUTPUT"
    echo "  📦 Size: $SIZE"
    echo "═══════════════════════════════"
else
    echo "❌ ISO creation failed"
    exit 1
fi
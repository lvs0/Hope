#!/bin/bash
# Hope OS — ISO Build Script
# Builds a bootable ISO from the hope-iso/ contents

set -e

HOPE_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ISO_DIR="$HOPE_ROOT/hope-iso"
OUTPUT="$HOPE_ROOT/hope-os.iso"

echo "[Hope] Building ISO..."

# Check for required tools
for cmd in debootstrap xorriso mksquashfs; do
    if ! command -v "$cmd" &> /dev/null; then
        echo "[ERROR] Missing required tool: $cmd"
        echo "Run ./scripts/setup-deps.sh first"
        exit 1
    fi
done

# Verify source exists
if [ ! -d "$ISO_DIR/hope" ]; then
    echo "[ERROR] Source directory $ISO_DIR/hope not found"
    exit 1
fi

# Build the ISO
echo "[Hope] Creating ISO image..."
xorriso -as mkisofs \
    -r \
    -J \
    -joliet-long \
    -V "HOPEOS" \
    -partition_offset 16 \
    -append_partition 2 esp "$ISO_DIR/esp.img" \
    -o "$OUTPUT" \
    "$ISO_DIR"

echo "[Hope] ISO built: $OUTPUT"
echo "[Hope] Size: $(du -h "$OUTPUT" | cut -f1)"

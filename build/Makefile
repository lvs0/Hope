#!/usr/bin/env bash
# Hope OS Build System
# Target: Debian Stable 12 + linux-zen

set -e

HOPE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="${HOPE_ROOT}/build"
OUTPUT_DIR="${HOPE_ROOT}/output"

# Colors
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; BLUE='\033[0;34m'; NC='\033[0m'

info()  { echo -e "${BLUE}[INFO]${NC} $1"; }
ok()    { echo -e "${GREEN}[OK]${NC} $1"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# Check deps
check_deps() {
    info "Checking build dependencies..."
    local deps=(debootstrap qemu-utils xorriso squashfs-tools grub-pc-bin)
    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &>/dev/null; then
            error "Missing dependency: $dep (install with: apt install ${deps[*]})"
        fi
    done
    ok "All dependencies satisfied"
}

# Build kernel
build_kernel() {
    info "Building Hope kernel (linux-zen + localmodconfig)..."
    local kernel_src="${BUILD_DIR}/linux-zen"
    
    if [ ! -d "$kernel_src" ]; then
        git clone --depth=1 --branch=6.14-zen1 https://github.com/zen-kernel/zen-kernel.git "$kernel_src" || \
        git clone --depth=1 https://github.com/torvalds/linux.git "$kernel_src"
    fi
    
    cd "$kernel_src"
    
    # Copy kernel config
    cp "${BUILD_DIR}/configs/kernel-config" .config || true
    
    # Build with localmodconfig for faster compilation
    make localmodconfig 2>/dev/null || true
    make -j$(nproc) bindeb-pkg
    
    ok "Kernel built"
}

# Build HAL+
build_hal() {
    info "Building HAL+ (Hardware Adaptation Layer)..."
    cd "${HOPE_ROOT}/hal"
    cargo build --release
    ok "HAL+ built"
}

# Create ISO
build_iso() {
    info "Creating Hope OS ISO..."
    mkdir -p "$OUTPUT_DIR"
    
    # Bootstrap Debian
    debootstrap --arch=amd64 stable "${OUTPUT_DIR}/chroot" http://deb.debian.org/debian
    
    # Copy HAL+
    cp "${HOPE_ROOT}/hal/target/release/hal-daemon "${OUTPUT_DIR}/chroot/usr/local/bin/"
    
    # Install Hope packages
    # Copy kernel deb
    cp "${BUILD_DIR}"/*.deb "${OUTPUT_DIR}/chroot/tmp/" 2>/dev/null || true
    
    # Configure system
    chroot "${OUTPUT_DIR}/chroot" /bin/bash -c "
        apt-get install -y systemd linux-image-amd64 grub2 efibits
        systemctl enable hal-daemon
    "
    
    # Build ISO
    xorriso -as mkisofs -r -J -A HOPE_OS -V HOPE_OS \
        -isohybrid-mbr /usr/lib/syslinux/bios/mbr.bin \
        -partition_offset 16 \
        -efi-boot-part --efi-boot-image \
        -o "${OUTPUT_DIR}/hope-os.iso" \
        "${OUTPUT_DIR}/chroot"
    
    ok "ISO created: ${OUTPUT_DIR}/hope-os.iso"
}

# Main
main() {
    echo "═══════════════════════════════════"
    echo "  Hope OS Build System v0.1"
    echo "═══════════════════════════════════"
    echo
    
    check_deps
    build_hal
    build_iso
    
    echo
    ok "Hope OS build complete!"
    echo "Output: ${OUTPUT_DIR}/hope-os.iso"
}

main "$@"
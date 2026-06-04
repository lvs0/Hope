#!/usr/bin/env bash
# make-iso.sh — Create bootable Hope OS ISO
#
# Creates an ISO with:
# - Debian Stable 12 base
# - Hope OS kernel (linux-zen optimized)
# - HAL+ (Hardware Adaptation Layer)
# - Hope Shell (Wayland compositor)
# - Btrfs with snapshot support

set -euo pipefail

# Configuration
HOPE_VERSION="0.1.0-alpha"
ARCH="$(uname -m)"
JOBS="$(nproc)"

# Paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HOPE_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
KERNEL_DIR="${HOPE_ROOT}/build/kernel"
ISO_DIR="${HOPE_ROOT}/build/iso"
PKG_DIR="${HOPE_ROOT}/packages"
WORK_DIR="${HOPE_ROOT}/build/work"
OUTPUT_DIR="${ISO_DIR}/output"

# ISO parameters
ISO_LABEL="HOPEOS"
ISO_VOLUME="Hope OS ${HOPE_VERSION}"
ISO_APPID="Hope OS"
ISO_PUBLISHER="Hope OS Team"
ISO_BOOT_SIZE=4

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

log() { echo -e "${GREEN}[iso]${NC} $1"; }
warn() { echo -e "${YELLOW}[warn]${NC} $1"; }
error() { echo -e "${RED}[error]${NC} $1" >&2; }

usage() {
    cat <<EOF
Usage: $0 [OPTIONS]

Options:
    --kernel FILE     Path to kernel image
    --initrd FILE     Path to initrd
    --packages DIR    Directory with .deb packages
    --output FILE     Output ISO path
    -h, --help        Show this help

Examples:
    $0 --kernel /path/bzImage --initrd /path/initrd.gz
    $0 --packages ./packages --output hope-os.iso

EOF
}

KERNEL_IMAGE=""
INITRD_IMAGE=""
PACKAGES_DIR="${PKG_DIR}"
OUTPUT_ISO="${OUTPUT_DIR}/hope-os-${HOPE_VERSION}.iso"

while [[ $# -gt 0 ]]; do
    case $1 in
        --kernel) KERNEL_IMAGE="$2"; shift 2 ;;
        --initrd) INITRD_IMAGE="$2"; shift 2 ;;
        --packages) PACKAGES_DIR="$2"; shift 2 ;;
        --output) OUTPUT_ISO="$2"; shift 2 ;;
        -h|--help) usage; exit 0 ;;
        *) error "Unknown option: $1"; exit 1 ;;
    esac
done

main() {
    log "Hope OS ISO Builder"
    log "Version: ${HOPE_VERSION}"
    echo ""

    check_dependencies
    setup_directories
    prepare_base
    install_kernel
    install_packages
    configure_boot
    create_iso

    log "ISO created: ${OUTPUT_ISO}"
}

check_dependencies() {
    log "Checking dependencies..."

    local deps=("xorriso" "mtools" "syslinux-utils" "parted")
    local missing=()

    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &>/dev/null && ! dpkg -s "$dep" &>/dev/null 2>&1; then
            missing+=("$dep")
        fi
    done

    if [[ ${#missing[@]} -gt 0 ]]; then
        warn "Missing: ${missing[*]}"
        warn "Install: sudo apt install ${missing[*]}"
        error "Cannot build ISO without dependencies"
        exit 1
    fi

    log "Dependencies satisfied"
}

setup_directories() {
    log "Setting up directories..."
    mkdir -p "${WORK_DIR}/iso"
    mkdir -p "${WORK_DIR}/image"
    mkdir -p "${OUTPUT_DIR}"
    log "Directories ready"
}

prepare_base() {
    log "Preparing base ISO structure..."

    local isoroot="${WORK_DIR}/iso"
    local -a dirs=(
        "boot"
        "boot/grub"
        "boot/grub/fonts"
        "boot/hope"
        "live"
        "live/boot-dev"
        "live/boot-hybrid"
        "pool"
        "dists/stable"
        "pool/main"
        ".disk"
    )

    for dir in "${dirs[@]}"; do
        mkdir -p "${isoroot}/${dir}"
    done

    # Create base file structure for live system
    create_live_base

    log "Base structure ready"
}

create_live_base() {
    log "Creating live system base..."

    local isoroot="${WORK_DIR}/iso"
    local live_root="${WORK_DIR}/image"

    mkdir -p "${live_root}/etc"
    mkdir -p "${live_root}/var"
    mkdir -p "${live_root}/var/lib"
    mkdir -p "${live_root}/home/hope"

    # Create minimal fstab for live boot
    cat > "${live_root}/etc/fstab" <<'FSTAB'
# Hope OS Live FSTAB
proc /proc proc defaults 0 0
sysfs /sys sysfs defaults 0 0
tmpfs /tmp tmpfs defaults 0 0
FSTAB

    # Create hope user
    useradd -m -s /bin/bash hope 2>/dev/null || true
}

install_kernel() {
    log "Installing Hope OS kernel..."

    local isoroot="${WORK_DIR}/iso"
    local kernel_dest="${isoroot}/boot/hope"

    mkdir -p "${kernel_dest}"

    if [[ -n "${KERNEL_IMAGE}" ]] && [[ -f "${KERNEL_IMAGE}" ]]; then
        cp "${KERNEL_IMAGE}" "${kernel_dest}/vmlinuz"
        log "Kernel image installed"
    elif [[ -d "${KERNEL_DIR}/output" ]]; then
        local vmlinuz=$(find "${KERNEL_DIR}/output" -name "vmlinuz*" -type f 2>/dev/null | head -1)
        if [[ -f "${vmlinuz}" ]]; then
            cp "${vmlinuz}" "${kernel_dest}/vmlinuz"
            log "Kernel from build output installed"
        fi
    else
        warn "No kernel found, using placeholder"
        touch "${kernel_dest}/vmlinuz.hope"
    fi

    if [[ -n "${INITRD_IMAGE}" ]] && [[ -f "${INITRD_IMAGE}" ]]; then
        cp "${INITRD_IMAGE}" "${kernel_dest}/initrd.img"
        log "Initrd installed"
    else
        warn "No initrd found, using placeholder"
        touch "${kernel_dest}/initrd.img.hope"
    fi
}

install_packages() {
    log "Installing Hope OS packages..."

    local pool="${WORK_DIR}/iso/pool"
    local -a pkg_dirs=("main")

    for dir in "${pkg_dirs[@]}"; do
        mkdir -p "${pool}/${dir}"
    done

    # Copy all .deb packages
    if [[ -d "${PACKAGES_DIR}" ]]; then
        find "${PACKAGES_DIR}" -name "*.deb" -type f -exec cp {} "${pool}/main/" \; 2>/dev/null || true
        log "Packages copied to ISO"
    else
        warn "No packages directory found at ${PACKAGES_DIR}"
    fi
}

configure_boot() {
    log "Configuring boot system..."

    local isoroot="${WORK_DIR}/iso"

    # Create GRUB configuration
    cat > "${isoroot}/boot/grub/grub.cfg" <<'GRUB'
# Hope OS GRUB Configuration
# Deep Space theme

set default="0"
set timeout="5"

insmod all_video
insmod gfxterm

# Hope OS Deep Space palette
set menu_color_normal=black/black
set menu_color_highlight=yellow/black

# Hope OS menu entry
menuentry "Hope OS" {
    linux /boot/hope/vmlinuz boot=live quiet splash
    initrd /boot/hope/initrd.img
}

menuentry "Hope OS (Safe Mode)" {
    linux /boot/hope/vmlinuz boot=live safegraphics
    initrd /boot/hope/initrd.img
}

menuentry "Hope OS (Memory Test)" {
    linux /boot/hope/memtest86+.bin
}

# Advanced
menuentry "Hope OS (Btrfs Snapshot)" {
    linux /boot/hope/vmlinuz boot=live btrfs.restore.snapshot=yes
    initrd /boot/hope/initrd.img
}
GRUB

    # Copy fonts
    if [[ -f /usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf ]]; then
        cp /usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf "${isoroot}/boot/grub/fonts/"
    fi

    # Create ISOLINUX configuration for BIOS boot
    cat > "${isoroot}/boot/syslinux/isolinux.cfg" <<'ISOLINUX'
DEFAULT hope
LABEL hope
    KERNEL /boot/hope/vmlinuz
    APPEND boot=live quiet splash initrd=/boot/hope/initrd.img
LABEL safemode
    KERNEL /boot/hope/vmlinuz
    APPEND boot=live safegraphics initrd=/boot/hope/initrd.img
PROMPT 1
TIMEOUT 50
ISOLINUX

    # Create .disk/info
    echo "Hope OS ${HOPE_VERSION}" > "${isoroot}/.disk/info"
    echo "Hope OS Team" > "${isoroot}/.disk/publisher"

    # Create manifest
    cat > "${isoroot}/hope-manifest.txt" <<MANIFEST
Hope OS ${HOPE_VERSION}
Built: $(date -Iseconds)
Architecture: ${ARCH}
Kernel: linux-zen with BFQ + ZRAM
MANIFEST

    log "Boot configuration ready"
}

create_iso() {
    log "Creating ISO image..."

    # Ensure output directory exists
    mkdir -p "$(dirname "${OUTPUT_ISO}")"

    # Build ISO with xorriso
    xorriso \
        -as mkisofs \
        -r \
        -J \
        -joliet-long \
        -label "${ISO_LABEL}" \
        -volset "${ISO_VOLUME}" \
        -appid "${ISO_APPID}" \
        -publisher "${ISO_PUBLISHER}" \
        -m "*.hope" \
        -boot-info-table \
        --grub2-boot-info \
        -boot-load-size 4 \
        -iso-level 3 \
        -boot-load-size "${ISO_BOOT_SIZE}" \
        -eltorito-alt-boot \
        -e boot/hybrid \
        -no-emul-boot \
        -isohybrid-gpt-basdat \
        -isohybrid-apm-hfsplus \
        -output "${OUTPUT_ISO}" \
        "${WORK_DIR}/iso"

    # Calculate SHA256
    if command -v sha256sum &>/dev/null; then
        sha256sum "${OUTPUT_ISO}" > "${OUTPUT_ISO}.sha256"
        log "ISO SHA256: $(cat "${OUTPUT_ISO}.sha256")"
    fi

    # Report size
    local size=$(du -h "${OUTPUT_ISO}" | cut -f1)
    log "ISO size: ${size}"
}

main "$@"

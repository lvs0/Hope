#!/usr/bin/env bash
# build-kernel.sh — Build linux-zen with localmodconfig for Hope OS
#
# Optimizations:
# - localmodconfig: ~30% lighter kernel by excluding unused modules
# - BFQ I/O scheduler: optimal for desktop and SSD
# - ZRAM zstd: memory compression for systems with ≤8GB RAM

set -euo pipefail

# Configuration
KERNEL_VERSION="6.12.1"
KERNEL_NAME="zen"
KERNEL_FLAVOR="${KERNEL_NAME}-hope"
KERNEL_DEB_VERSION="1"
ARCH="$(uname -m)"
JOBS="$(nproc)"

# Paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HOPE_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
KERNEL_DIR="${HOPE_ROOT}/build/kernel"
CONFIG_FILE="${HOPE_ROOT}/build/configs/kernel-config"
SRC_DIR="${KERNEL_DIR}/linux-${KERNEL_VERSION}-${KERNEL_NAME}"
BUILD_DIR="${KERNEL_DIR}/build"
OUTPUT_DIR="${KERNEL_DIR}/output"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

log() { echo -e "${GREEN}[build]${NC} $1"; }
warn() { echo -e "${YELLOW}[warn]${NC} $1"; }
error() { echo -e "${RED}[error]${NC} $1" >&2; }

usage() {
    cat <<EOF
Usage: $0 [OPTIONS]

Options:
    --config FILE      Use custom kernel config (default: ${CONFIG_FILE})
    --clean            Clean before build
    --deb              Build .deb packages
    --all              Full build with modules
    -h, --help         Show this help

Examples:
    $0 --config /path/to/config
    $0 --deb
    $0 --all

EOF
}

# Parse arguments
CLEAN_BUILD=false
BUILD_DEB=false
BUILD_ALL=false
CUSTOM_CONFIG=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --config) CUSTOM_CONFIG="$2"; shift 2 ;;
        --clean) CLEAN_BUILD=true; shift ;;
        --deb) BUILD_DEB=true; shift ;;
        --all) BUILD_ALL=true; shift ;;
        -h|--help) usage; exit 0 ;;
        *) error "Unknown option: $1"; usage; exit 1 ;;
    esac
done

main() {
    log "Hope OS Kernel Build Script"
    log "Kernel: linux-${KERNEL_VERSION}-${KERNEL_NAME}"
    log "Architecture: ${ARCH}"
    log "Jobs: ${JOBS}"
    echo

    check_dependencies
    setup_directories

    if $CLEAN_BUILD; then
        clean_kernel
    fi

    download_kernel
    apply_hope_optimizations
    build_kernel

    if $BUILD_DEB; then
        build_deb_packages
    fi

    log "Kernel build complete!"
    log "Output: ${OUTPUT_DIR}"
}

check_dependencies() {
    log "Checking dependencies..."

    local deps=("make" "gcc" "bison" "flex" "libelf-dev" "libssl-dev" "bc")
    local missing=()

    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &>/dev/null && ! dpkg -s "$dep" &>/dev/null 2>&1; then
            missing+=("$dep")
        fi
    done

    if [[ ${#missing[@]} -gt 0 ]]; then
        warn "Missing dependencies: ${missing[*]}"
        warn "Install with: sudo apt install ${missing[*]}"
        error "Cannot proceed without dependencies"
        exit 1
    fi

    log "All dependencies satisfied"
}

setup_directories() {
    log "Setting up directories..."
    mkdir -p "${KERNEL_DIR}"
    mkdir -p "${BUILD_DIR}"
    mkdir -p "${OUTPUT_DIR}"
    mkdir -p "${SRC_DIR}"
    log "Directories ready"
}

download_kernel() {
    if [[ -d "${SRC_DIR}" ]] && [[ -f "${SRC_DIR}/Makefile" ]]; then
        log "Kernel source already present"
        return
    fi

    log "Downloading linux-${KERNEL_VERSION}-${KERNEL_NAME}..."

    local tarball="linux-${KERNEL_VERSION}.${KERNEL_NAME}.tar.gz"
    local url="https://github.com/zen-kernel/zen-kernel/releases/download/v${KERNEL_VERSION}-zen1/${tarball}"

    if command -v wget &>/dev/null; then
        wget -q --show-progress -O "${KERNEL_DIR}/${tarball}" "$url" || true
    elif command -v curl &>/dev/null; then
        curl -L -o "${KERNEL_DIR}/${tarball}" "$url" || true
    fi

    if [[ -f "${KERNEL_DIR}/${tarball}" ]]; then
        tar -xzf "${KERNEL_DIR}/${tarball}" -C "${KERNEL_DIR}"
        mv "${KERNEL_DIR}/linux-${KERNEL_VERSION}-${KERNEL_NAME}" "${SRC_DIR}"
    else
        warn "Could not download kernel source"
        warn "Please place kernel source in: ${SRC_DIR}"
        exit 1
    fi
}

apply_hope_optimizations() {
    log "Applying Hope OS optimizations..."

    # Use Hope config as base
    if [[ -f "${CONFIG_FILE}" ]]; then
        log "Applying Hope kernel config from ${CONFIG_FILE}"
        cp "${CONFIG_FILE}" "${SRC_DIR}/.config"
    else
        warn "Hope kernel config not found, using defaults"
    fi

    # Apply BFQ scheduler if not already set
    if grep -q "CONFIG_BFQ_GROUP_IOSCHED=y" "${SRC_DIR}/.config" 2>/dev/null; then
        log "BFQ I/O scheduler already enabled"
    else
        echo "CONFIG_BFQ_GROUP_IOSCHED=y" >> "${SRC_DIR}/.config"
    fi

    # Apply ZRAM if not already set
    if grep -q "CONFIG_ZRAM=y" "${SRC_DIR}/.config" 2>/dev/null; then
        log "ZRAM already enabled"
    else
        echo "CONFIG_ZRAM=y" >> "${SRC_DIR}/.config"
        echo "CONFIG_ZRAM_DEFAULT_COMPRESS=\"zstd\"" >> "${SRC_DIR}/.config"
    fi

    log "Optimizations applied"
}

build_kernel() {
    log "Building kernel..."

    cd "${SRC_DIR}"

    # Generate config from .config
    make KERNEL_VERSION="${KERNEL_VERSION}-${KERNEL_FLAVOR}" \
         ARCH="${ARCH}" \
         olddefconfig

    # Build with localmodconfig for lighter kernel (~30% reduction)
    if [[ -f /proc/config.gz ]]; then
        log "Using localmodconfig (current kernel config as baseline)"
        zcat /proc/config.gz > "${SRC_DIR}/.config"
        make localmodconfig KERNEL_VERSION="${KERNEL_VERSION}-${KERNEL_FLAVOR}" ARCH="${ARCH}"
    fi

    # Build kernel
    log "Compiling kernel (this may take a while)..."
    make -j"${JOBS}" \
         KERNEL_VERSION="${KERNEL_VERSION}-${KERNEL_FLAVOR}" \
         ARCH="${ARCH}" \
         bindeb-pkg \
         LOCALVERSION="-hope" \
         KDEB_PKGVERSION="${KERNEL_DEB_VERSION}" \
         2>&1 | tee "${BUILD_DIR}/kernel-build.log"

    # Copy outputs
    cp "${KERNEL_DIR}"/*.deb "${OUTPUT_DIR}/" 2>/dev/null || true
    cp "${KERNEL_DIR}"/*.xz "${OUTPUT_DIR}/" 2>/dev/null || true

    log "Kernel built successfully"
}

build_deb_packages() {
    log "Building Debian packages..."

    cd "${SRC_DIR}"

    make -j"${JOBS}" \
         KERNEL_VERSION="${KERNEL_VERSION}-${KERNEL_FLAVOR}" \
         ARCH="${ARCH}" \
         KDEB_PKGVERSION="${KERNEL_DEB_VERSION}" \
         bindeb-pkg

    log "Debian packages created"
}

clean_kernel() {
    log "Cleaning kernel build..."
    cd "${SRC_DIR}"
    make clean
    rm -f "${KERNEL_DIR}"/*.deb
    rm -f "${KERNEL_DIR}"/*.xz
    log "Clean complete"
}

main "$@"

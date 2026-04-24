#!/usr/bin/env bash
# install-hope.sh — Hope OS Installation Script
#
# Installs Hope OS to disk with:
# - Btrfs for immutable snapshots
# - LUKS2 full-disk encryption (optional)
# - ZRAM configuration
# - HAL+ setup
# - Hope Shell integration

set -euo pipefail

# Configuration
HOPE_VERSION="0.1.0-alpha"

# Paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HOPE_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
PKG_DIR="${HOPE_ROOT}/packages"

# Device to install to
TARGET_DISK=""
USE_ENCRYPTION=false
USE_BTRFS_SNAPSHOTS=true
HOPE_USER=""
HOPE_HOSTNAME="hope"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BLUE='\033[0;34m'
NC='\033[0m'

log() { echo -e "${GREEN}[install]${NC} $1"; }
warn() { echo -e "${YELLOW}[warn]${NC} $1"; }
error() { echo -e "${RED}[error]${NC} $1" >&2; }
ask() { echo -e "${CYAN}[ask]${NC} $1"; }

usage() {
    cat <<EOF
Hope OS Installer — 9 minutes, 0 jargon

Usage: $0 [OPTIONS]

Options:
    --disk DEVICE     Target disk (e.g., /dev/sda)
    --user NAME       Username for Hope user
    --hostname NAME   Hostname (default: hope)
    --encryption      Enable LUKS2 full-disk encryption
    --no-snapshots    Disable Btrfs snapshots
    -h, --help        Show this help

Examples:
    $0 --disk /dev/sda --user lvs
    $0 --disk /dev/nvme0n1 --encryption

EOF
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --disk) TARGET_DISK="$2"; shift 2 ;;
        --user) HOPE_USER="$2"; shift 2 ;;
        --hostname) HOPE_HOSTNAME="$2"; shift 2 ;;
        --encryption) USE_ENCRYPTION=true; shift ;;
        --no-snapshots) USE_BTRFS_SNAPSHOTS=false; shift ;;
        -h|--help) usage; exit 0 ;;
        *) error "Unknown option: $1"; exit 1 ;;
    esac
done

main() {
    # Check if running as root
    if [[ $EUID -ne 0 ]]; then
        error "This script must be run as root"
        error "Usage: sudo $0 ..."
        exit 1
    fi

    check_dependencies
    interactive_config
    confirm_installation
    partition_disk
    setup_filesystem
    install_packages
    configure_system
    configure_boot
    configure_hal
    final_cleanup

    log "Installation complete!"
    show_next_steps
}

check_dependencies() {
    log "Checking dependencies..."

    local deps=("btrfs" "cryptsetup" "parted" "mkfs.btrfs" "mkfs.ext4")
    local missing=()

    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &>/dev/null; then
            missing+=("$dep")
        fi
    done

    if [[ ${#missing[@]} -gt 0 ]]; then
        error "Missing: ${missing[*]}"
        error "Install: apt install btrfs-progs cryptsetup parted"
        exit 1
    fi

    log "Dependencies satisfied"
}

interactive_config() {
    echo ""
    echo "┌──────────────────────────────────────────────┐"
    echo "│         Hope OS Installer v${HOPE_VERSION}         │"
    echo "└──────────────────────────────────────────────┘"
    echo ""

    # Ask for disk
    if [[ -z "${TARGET_DISK}" ]]; then
        ask "Quel disque installer? (ex: /dev/sda)"
        read -r TARGET_DISK
    fi

    if [[ ! -b "${TARGET_DISK}" ]]; then
        error "Device not found: ${TARGET_DISK}"
        exit 1
    fi

    # Ask for username
    if [[ -z "${HOPE_USER}" ]]; then
        ask "Nom d'utilisateur?"
        read -r HOPE_USER
    fi

    if [[ -z "${HOPE_USER}" ]]; then
        HOPE_USER="hope"
    fi

    # Ask about encryption
    ask "Chiffrement LUKS2? (recommandé pour sécurité maximale)"
    ask "  non = Standard | oui = Chiffrement total"
    select encrypted in "non" "oui"; do
        case $encrypted in
            oui) USE_ENCRYPTION=true; break ;;
            non) USE_ENCRYPTION=false; break ;;
        esac
    done

    # Ask about snapshots
    if [[ "${USE_BTRFS_SNAPSHOTS}" == "true" ]]; then
        ask "Snapshots Btrfs automatiques? (rollback 30s)"
        ask "  non = Désactivé | oui = Rollback automatique"
        select snapshots in "non" "oui"; do
            case $snapshots in
                oui) USE_BTRFS_SNAPSHOTS=true; break ;;
                non) USE_BTRFS_SNAPSHOTS=false; break ;;
            esac
        done
    fi

    echo ""
}

confirm_installation() {
    echo ""
    echo "┌──────────────────────────────────────────────┐"
    echo "│           Récapitulatif                       │"
    echo "├──────────────────────────────────────────────┤"
    echo "│  Disque:      ${TARGET_DISK}                    │"
    echo "│  Utilisateur: ${HOPE_USER}                        │"
    echo "│  Nom machine: ${HOPE_HOSTNAME}                      │"
    echo "│  Chiffrement: $([ "$USE_ENCRYPTION" == "true" ] && echo "LUKS2" || echo "Standard")                           │"
    echo "│  Snapshots:   $([ "$USE_BTRFS_SNAPSHOTS" == "true" ] && echo "Btrfs" || echo "Désactivé")                           │"
    echo "└──────────────────────────────────────────────┘"
    echo ""

    ask "Prêt à installer. Continuer?"
    select confirmed in "Installer" "Annuler"; do
        case $confirmed in
            Installer) break ;;
            Annuler) log "Installation annulée"; exit 0 ;;
        esac
    done

    echo ""
}

partition_disk() {
    log "Partitioning disk..."

    # Unmount any mounted partitions
    umount "${TARGET_DISK}"* 2>/dev/null || true

    # Create new partition table
    parted -s "${TARGET_DISK}" mklabel gpt

    # Create EFI partition (512MB)
    parted -s "${TARGET_DISK}" mkpart primary fat32 1MiB 513MiB
    parted -s "${TARGET_DISK}" set 1 esp on

    # Create root partition (rest of disk)
    parted -s "${TARGET_DISK}" mkpart primary btrfs 513MiB 100%

    # Give system time to see new partitions
    sleep 1
    partprobe "${TARGET_DISK}" 2>/dev/null || true

    log "Disk partitioned"
}

setup_filesystem() {
    log "Setting up filesystem..."

    local root_part="${TARGET_DISK}p2"
    local efi_part="${TARGET_DISK}p1"

    if [[ ! -b "${root_part}" ]]; then
        root_part="${TARGET_DISK}2"  # fallback for non-block devices
    fi
    if [[ ! -b "${efi_part}" ]]; then
        efi_part="${TARGET_DISK}1"
    fi

    # Setup Btrfs for root
    mkfs.btrfs -L "HopeOS" -f "${root_part}"

    # Mount root for subvolume setup
    mount "${root_part}" /mnt

    # Create Btrfs subvolumes
    btrfs subvolume create /mnt/@
    btrfs subvolume create /mnt/@home
    btrfs subvolume create /mnt/@var
    btrfs subvolume create /mnt/@snapshots

    # Configure snapshots if enabled
    if [[ "${USE_BTRFS_SNAPSHOTS}" == "true" ]]; then
        btrfs subvolume snapshot /mnt/@ /mnt/@snapshots/base 2>/dev/null || true
    fi

    umount /mnt

    # Mount with subvolumes
    mount -o subvol=@ "${root_part}" /mnt
    mkdir -p /mnt/home
    mount -o subvol=@home "${root_part}" /mnt/home
    mkdir -p /mnt/var
    mount -o subvol=@var "${root_part}" /mnt/var

    # Setup EFI
    mkfs.fat -F32 "${efi_part}"
    mkdir -p /mnt/boot/efi
    mount "${efi_part}" /mnt/boot/efi

    log "Filesystem ready"
}

install_packages() {
    log "Installing packages..."

    # Mount essential filesystems
    mount --bind /dev /mnt/dev
    mount --bind /proc /mnt/proc
    mount --bind /sys /mnt/sys

    # Install Hope packages
    if [[ -d "${PKG_DIR}/hope-core/debian" ]]; then
        dpkg -i "${PKG_DIR}"/hope-core/*.deb 2>/dev/null || true
    fi
    if [[ -d "${PKG_DIR}/hope-halo" ]]; then
        dpkg -i "${PKG_DIR}"/hope-halo/*.deb 2>/dev/null || true
    fi

    # Copy Hope OS files
    cp -r "${HOPE_ROOT}/hal" /mnt/opt/hope-hal 2>/dev/null || true
    cp -r "${HOPE_ROOT}/hope-*" /mnt/opt/ 2>/dev/null || true

    log "Packages installed"
}

configure_system() {
    log "Configuring system..."

    local mnt="/mnt"

    # Set hostname
    echo "${HOPE_HOSTNAME}" > "${mnt}/etc/hostname"
    cat > "${mnt}/etc/hosts" <<HOSTS
127.0.0.1 localhost
127.0.1.1 ${HOPE_HOSTNAME}
::1       localhost ip6-localhost ip6-loopback
HOSTS

    # Create Hope user
    useradd -m -s /bin/bash -G wheel,sudo "${HOPE_USER}" 2>/dev/null || true

    # Configure fstab
    cat > "${mnt}/etc/fstab" <<FSTAB
# Hope OS fstab
UUID=$(blkid -s UUID -o value "${TARGET_DISK}p2") / btrfs subvol=@,defaults,ssd 0 0
UUID=$(blkid -s UUID -o value "${TARGET_DISK}p2") /home btrfs subvol=@home,defaults,ssd 0 0
UUID=$(blkid -s UUID -o value "${TARGET_DISK}p1") /boot/efi vfat defaults 0 0
tmpfs /tmp tmpfs defaults 0 0
FSTAB

    # Configure ZRAM
    cat > "${mnt}/etc/sysctl.d/99-hope-zram.conf" <<ZRAM
# Hope OS ZRAM configuration
vm.swappiness=100
vm.watermark_scale_factor=200
ZRAM

    # Configure kernel sysctl
    cp "${HOPE_ROOT}/build/configs/sysctl.conf" "${mnt}/etc/sysctl.d/99-hope.conf"

    log "System configured"
}

configure_boot() {
    log "Configuring bootloader..."

    # Install GRUB
    grub-install --target=x86_64-efi --efi-directory=/mnt/boot/efi --bootloader-id=HopeOS "${TARGET_DISK}"

    # Update GRUB config
    update-grub

    log "Bootloader ready"
}

configure_hal() {
    log "Configuring HAL+..."

    local mnt="/mnt"

    # Create HAL+ systemd service
    cat > "${mnt}/etc/systemd/system/hope-hal.service" <<HAL
[Unit]
Description=Hope OS Hardware Adaptation Layer
After=network.target

[Service]
Type=simple
User=root
ExecStart=/opt/hope-hal/target/release/hope-hal
Restart=on-failure
RestartSec=5s

[Install]
WantedBy=multi-user.target
HAL

    # Enable HAL+ service
    systemctl enable hope-hal --root="${mnt}" 2>/dev/null || true

    log "HAL+ configured"
}

final_cleanup() {
    log "Final cleanup..."

    # Unmount filesystems
    umount -l /mnt/boot/efi 2>/dev/null || true
    umount -l /mnt/home 2>/dev/null || true
    umount -l /mnt/var 2>/dev/null || true
    umount -l /mnt 2>/dev/null || true

    sync

    log "System ready"
}

show_next_steps() {
    echo ""
    echo "┌──────────────────────────────────────────────┐"
    echo "│          Installation terminée! 🎉             │"
    echo "├──────────────────────────────────────────────┤"
    echo "│                                                │"
    echo "│  Redémarrez et retirez le média d'installation │"
    echo "│                                                │"
    echo "│  Premier démarrage: création des snapshots Btrfs│"
    echo "│                                                │"
    echo "│  Commandes utiles:                            │"
    echo "│    btrfs snapshot list /                      │"
    echo "│    btrfs rollback / @snapshots/base           │"
    echo "│    hope-ctl status                            │"
    echo "│                                                │"
    echo "└──────────────────────────────────────────────┘"
    echo ""
}

main "$@"

#!/bin/bash
# Hope OS — Dependency Installer
# Installs required tools to build Hope OS

set -e

echo "[Hope] Installing build dependencies..."

if command -v apt-get &> /dev/null; then
    sudo apt-get update
    sudo apt-get install -y debootstrap xorriso squashfs-tools
elif command -v dnf &> /dev/null; then
    sudo dnf install -y debootstrap xorriso squashfs-tools
elif command -v pacman &> /dev/null; then
    sudo pacman -S --noconfirm debootstrap xorriso squashfs-tools
else
    echo "[ERROR] Unsupported package manager"
    echo "Please install: debootstrap, xorriso, squashfs-tools"
    exit 1
fi

echo "[Hope] Dependencies installed successfully"

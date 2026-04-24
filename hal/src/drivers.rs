//! Driver installation logic
//!
//! Handles loading kernel modules and installing driver packages.

use log::{error, info, warn};
use std::process::Command;
use thiserror::Error;

use crate::db::DriverInfo;

/// Errors that can occur during driver installation
#[derive(Error, Debug)]
pub enum DriverError {
    #[error("modprobe failed: {0}")]
    Modprobe(String),
    #[error("package install failed: {0}")]
    PackageInstall(String),
    #[error("apt error: {0}")]
    Apt(String),
    #[error("module not found: {0}")]
    ModuleNotFound(String),
}

/// Install a driver based on driver info
pub fn install_driver(driver: &DriverInfo) -> Result<(), DriverError> {
    info!("Installing driver for {}", driver.name);

    // Load kernel module if needed
    if let Some(module) = &driver.kernel_module {
        load_kernel_module(module)?;
    }

    // Install firmware if needed
    if driver.needs_firmware {
        install_firmware()?;
    }

    // Install package if provided
    if let Some(package) = &driver.package {
        install_package(package)?;
    }

    // Apply configuration if provided
    if let Some(config) = &driver.config {
        apply_config(config)?;
    }

    info!("Driver {} installed successfully", driver.name);
    Ok(())
}

/// Load a kernel module via modprobe
fn load_kernel_module(module: &str) -> Result<(), DriverError> {
    info!("Loading kernel module: {}", module);

    let output = Command::new("modprobe")
        .arg(module)
        .output()
        .map_err(|e| DriverError::Modprobe(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("Module not found") || stderr.contains("no module") {
            return Err(DriverError::ModuleNotFound(module.to_string()));
        }
        return Err(DriverError::Modprobe(stderr.to_string()));
    }

    Ok(())
}

/// Install firmware packages if needed
fn install_firmware() -> Result<(), DriverError> {
    info!("Installing firmware packages");

    let output = Command::new("apt-get")
        .args(["install", "-y", "linux-firmware"])
        .output()
        .map_err(|e| DriverError::Apt(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("Firmware install warning: {}", stderr);
        // Don't fail entirely - firmware may already be present
    }

    Ok(())
}

/// Install a driver package via apt
fn install_package(package: &str) -> Result<(), DriverError> {
    info!("Installing package: {}", package);

    // Update package list first
    let update = Command::new("apt-get")
        .args(["update", "-qq"])
        .output()
        .map_err(|e| DriverError::Apt(e.to_string()))?;

    if !update.status.success() {
        warn!("apt-get update had issues");
    }

    let output = Command::new("apt-get")
        .args(["install", "-y", "-qq", package])
        .output()
        .map_err(|e| DriverError::Apt(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(DriverError::PackageInstall(stderr.to_string()));
    }

    Ok(())
}

/// Apply configuration (stub for now)
fn apply_config(config: &str) -> Result<(), DriverError> {
    info!("Applying driver config: {}", config);
    // TODO: Apply udev rules, sysctl settings, etc.
    Ok(())
}

/// Check if a module is currently loaded
pub fn is_module_loaded(module: &str) -> bool {
    let output = Command::new("lsmod")
        .output()
        .expect("lsmod failed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.lines().any(|line| line.starts_with(module))
}

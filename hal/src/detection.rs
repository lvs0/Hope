//! Device detection via udev
//!
//! Monitors udev for hardware events and parses vendor:product IDs.

use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::CString;
use thiserror::Error;

/// Errors that can occur in detection
#[derive(Error, Debug)]
pub enum DetectionError {
    #[error("udev init failed: {0}")]
    UdevInit(String),
    #[error("udev monitor failed: {0}")]
    UdevMonitor(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

/// A udev device event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UdevEvent {
    /// Event action (add, remove, change)
    pub action: String,
    /// Device subsystem (usb, pci, input, etc.)
    pub subsystem: String,
    /// USB vendor ID (4 hex chars)
    pub vendor_id: String,
    /// USB product ID (4 hex chars)
    pub product_id: String,
    /// Device sysfs path
    pub sysfs_path: Option<String>,
    /// Device name
    pub devname: Option<String>,
}

/// udev monitor for hardware events
pub struct UdevMonitor {
    #[allow(dead_code)]
    monitor: libudev::Monitor,
    #[allow(dead_code)]
    enumerator: libudev::Enumerator,
}

impl UdevMonitor {
    /// Create a new udev monitor
    pub fn new() -> Result<Self, DetectionError> {
        let context = libudev::Context::new();

        let monitor = context
            .monitor()
            .map_err(|e| DetectionError::UdevMonitor(e.to_string()))?;

        // Monitor USB and input subsystems for hotplug events
        monitor
            .match_subsystem("usb")
            .map_err(|e| DetectionError::UdevMonitor(e.to_string()))?;
        monitor
            .match_subsystem("input")
            .map_err(|e| DetectionError::UdevMonitor(e.to_string()))?;
        monitor
            .match_subsystem("sound")
            .map_err(|e| DetectionError::UdevMonitor(e.to_string()))?;

        let enumerator = libudev::Enumerator::new(&context)
            .map_err(|e| DetectionError::UdevInit(e.to_string()))?;

        // Add match for USB devices
        let _ = enumerator.match_subsystem("usb");

        Ok(Self {
            monitor,
            enumerator,
        })
    }

    /// Get the next udev event (non-blocking via polling)
    pub async fn next_event(&mut self) -> Option<UdevEvent> {
        // Poll for udev events every 500ms
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Use blocking udev API
        tokio::task::spawn_blocking(|| self.poll_udev())
            .await
            .ok()?
    }

    /// Poll udev for events (called from blocking thread)
    fn poll_udev(&self) -> Option<UdevEvent> {
        let context = libudev::Context::new();
        let mut monitor = match context.monitor() {
            Ok(m) => m,
            Err(_) => return None,
        };

        let _ = monitor.match_subsystem("usb");

        // Check for available events without blocking
        if let Ok(device) = monitor.receive_device() {
            return parse_udev_device(device);
        }

        None
    }
}

/// Parse a udev device into an event
fn parse_udev_device(device: libudev::Device) -> Option<UdevEvent> {
    let action = device.action()?.to_string();
    let subsystem = device.subsystem()?.to_string();

    // Extract vendor:product from ID_* fields
    let vendor_id = device
        .property_value("ID_VENDOR_ID")
        .map(|v| v.to_string())
        .unwrap_or_else(|| "0000".into());

    let product_id = device
        .property_value("ID_MODEL_ID")
        .map(|v| v.to_string())
        .unwrap_or_else(|| "0000".into());

    let sysfs_path = device
        .syspath()
        .file_name()
        .map(|s| s.to_string_lossy().to_string());

    let devname = device
        .devname()
        .map(|d| d.to_string_lossy().to_string());

    Some(UdevEvent {
        action,
        subsystem,
        vendor_id,
        product_id,
        sysfs_path,
        devname,
    })
}

/// Request a cloud lookup for an unknown device
pub async fn request_cloud_lookup(vendor_id: &str, product_id: &str) -> Result<(), DetectionError> {
    info!(
        "Requesting cloud lookup for unknown device {}:{}",
        vendor_id, product_id
    );

    // TODO: Implement actual cloud lookup via Polygone network
    // For now, just log and return
    debug!("Cloud lookup would contact: https://api.hope-os.org/drivers/lookup");

    Ok(())
}

/// Parse lsusb output to find a device
pub fn parse_lsusb_output(output: &str) -> Vec<UdevEvent> {
    let mut events = Vec::new();

    for line in output.lines() {
        // Format: Bus 001 Device 002: ID 8087:0a2a Intel Corp. Wireless-AC 9260
        let parts: Vec<&str> = line.splitn(4, ' ').collect();
        if parts.len() < 3 {
            continue;
        }

        // Look for ID pattern: 4hex:4hex
        let id_part = parts.get(2).unwrap_or(&"");
        if !id_part.contains(':') {
            continue;
        }

        let ids: Vec<&str> = id_part.split(':').collect();
        if ids.len() != 2 {
            continue;
        }

        let vendor_id = ids[0].to_lowercase();
        let product_id = ids[1].to_lowercase();

        events.push(UdevEvent {
            action: "add".into(),
            subsystem: "usb".into(),
            vendor_id,
            product_id,
            sysfs_path: None,
            devname: None,
        });
    }

    events
}

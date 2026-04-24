//! Device detection via udev
//!
//! Monitors udev for hardware events and parses vendor:product IDs.

use serde::{Deserialize, Serialize};
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
    pub action: String,
    pub subsystem: String,
    pub vendor_id: String,
    pub product_id: String,
    pub sysfs_path: Option<String>,
    pub devname: Option<String>,
}

/// udev monitor for hardware events
pub struct UdevMonitor;

impl UdevMonitor {
    /// Create a new udev monitor
    pub fn new() -> Result<Self, DetectionError> {
        Ok(UdevMonitor)
    }

    /// Scan for USB devices
    pub fn scan_usb_devices() -> Result<Vec<UdevEvent>, DetectionError> {
        let output = std::process::Command::new("lsusb")
            .output()
            .map_err(|e| DetectionError::Io(e))?;
        
        let text = String::from_utf8_lossy(&output.stdout);
        Ok(parse_lsusb_output(&text))
    }
}

/// Parse lsusb output to find a device
pub fn parse_lsusb_output(output: &str) -> Vec<UdevEvent> {
    let mut events = Vec::new();

    for line in output.lines() {
        let parts: Vec<&str> = line.splitn(4, ' ').collect();
        if parts.len() < 3 {
            continue;
        }

        let id_part = parts.get(2).unwrap_or(&"");
        if !id_part.contains(':') {
            continue;
        }

        let ids: Vec<&str> = id_part.split(':').collect();
        if ids.len() != 2 {
            continue;
        }

        events.push(UdevEvent {
            action: "add".into(),
            subsystem: "usb".into(),
            vendor_id: ids[0].to_lowercase(),
            product_id: ids[1].to_lowercase(),
            sysfs_path: None,
            devname: None,
        });
    }

    events
}

/// Request a cloud lookup for an unknown device
pub async fn request_cloud_lookup(vendor_id: &str, product_id: &str) -> Result<(), DetectionError> {
    tracing::info!(
        "Requesting cloud lookup for unknown device {}:{}",
        vendor_id, product_id
    );
    Ok(())
}

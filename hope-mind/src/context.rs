//! System context awareness for Hope Mind

use anyhow::Result;
use sysinfo::System;

/// System context information
#[derive(Debug, Clone)]
pub struct SystemContext {
    /// Total RAM in GB
    pub ram_total_gb: f64,
    /// Available RAM in GB
    pub ram_available_gb: f64,
    /// Number of CPU cores
    pub cpu_cores: usize,
    /// GPU information (if available)
    pub gpu_info: Option<String>,
}

impl SystemContext {
    /// Gather current system information
    pub fn gather() -> Result<Self> {
        let mut sys = System::new_all();
        sys.refresh_all();

        let ram_total = sys.total_memory() as f64 / 1_073_741_824.0; // bytes to GB
        let ram_available = sys.available_memory() as f64 / 1_073_741_824.0;
        let cpu_cores = sys.cpus().len();

        let gpu_info = detect_gpu();

        Ok(Self {
            ram_total_gb: ram_total,
            ram_available_gb: ram_available,
            cpu_cores,
            gpu_info,
        })
    }

    /// Get a formatted status string
    pub fn status_string(&self) -> String {
        format!(
            "RAM: {:.1}/{:.1} GB, CPU: {} cores{}",
            self.ram_total_gb - self.ram_available_gb,
            self.ram_total_gb,
            self.cpu_cores,
            self.gpu_info
                .as_ref()
                .map(|g| format!(", GPU: {}", g))
                .unwrap_or_default()
        )
    }
}

/// Detect GPU information
fn detect_gpu() -> Option<String> {
    // Try lspci for NVIDIA
    if let Ok(output) = std::process::Command::new("lspci")
        .args(["-v", "-s", "$(lspci | grep -i vga | cut -d' ' -f1)"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("NVIDIA") {
            return Some("NVIDIA (proprietary)".to_string());
        }
        if stdout.contains("AMD") || stdout.contains("ATI") {
            return Some("AMD/ATI".to_string());
        }
    }

    // Check for DRM
    if std::path::Path::new("/sys/class/drm").exists() {
        if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("card") && name.contains("-") {
                    continue; // Skip connector entries
                }
                if name.starts_with("card") {
                    return Some(format!("DRM: {}", name));
                }
            }
        }
    }

    None
}

/// Get OS context (running apps, active window, etc.)
pub fn get_os_context() -> String {
    let mut context = String::new();

    // Check if Wayland session
    if let Ok(display) = std::env::var("WAYLAND_DISPLAY") {
        context.push_str(&format!("Wayland: {}\n", display));
    }

    // Check if X11
    if let Ok(display) = std::env::var("DISPLAY") {
        context.push_str(&format!("X11: {}\n", display));
    }

    // Get hostname
    if let Ok(hostname) = std::env::var("HOSTNAME") {
        context.push_str(&format!("Host: {}\n", hostname));
    }

    // Get user
    if let Ok(user) = std::env::var("USER") {
        context.push_str(&format!("User: {}\n", user));
    }

    // Get uptime
    if let Ok(output) = std::process::Command::new("uptime").arg("-p").output() {
        let uptime = String::from_utf8_lossy(&output.stdout);
        context.push_str(&format!("Uptime: {}", uptime.trim()));
    }

    context
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gather_context() {
        let ctx = SystemContext::gather().unwrap();
        assert!(ctx.ram_total_gb > 0.0);
        assert!(ctx.cpu_cores > 0);
        println!("{}", ctx.status_string());
    }
}

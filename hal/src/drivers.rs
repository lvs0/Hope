//! Driver installation and management

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DriverError {
    #[error("Installation failed: {0}")]
    InstallFailed(String),
    #[error("Package not found: {0}")]
    PackageNotFound(String),
    #[error("Permission denied")]
    PermissionDenied,
}

/// Install a driver given its info
pub fn install_driver(driver_info: &crate::db::DriverInfo) -> Result<(), DriverError> {
    use std::process::Command;

    if let Some(module) = &driver_info.kernel_module {
        let output = Command::new("modprobe")
            .arg(module)
            .output()
            .map_err(|e| DriverError::InstallFailed(e.to_string()))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(DriverError::InstallFailed(err.to_string()));
        }
    }

    Ok(())
}

/// Check if a kernel module is loaded
pub fn is_module_loaded(name: &str) -> bool {
    use std::process::Command;
    
    let output = Command::new("lsmod")
        .output()
        .ok();

    output.map(|o| {
        let text = String::from_utf8_lossy(&o.stdout);
        text.lines().any(|l| l.starts_with(name))
    }).unwrap_or(false)
}

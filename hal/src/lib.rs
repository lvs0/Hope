//! Library entry point for hope-hal

pub mod db;
pub mod detection;
pub mod drivers;
pub mod notifications;

pub use db::{DriverDatabase, DriverInfo};
pub use detection::{parse_lsusb_output, UdevEvent, UdevMonitor};
pub use drivers::{install_driver, is_module_loaded};

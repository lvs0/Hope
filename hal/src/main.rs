//! Hope OS HAL+ — Hardware Adaptation Layer Daemon

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod db;
mod detection;
mod drivers;
mod notifications;

use db::DriverDatabase;
use detection::UdevMonitor;
use notifications::{spawn_notification_system, Notification};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Init logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Hope HAL+ starting...");

    // Init database
    let db = DriverDatabase::new()?;
    tracing::info!("Driver database initialized");

    // Spawn validated notification system
    let notifier = spawn_notification_system();

    tracing::info!("HAL+ running. Monitoring hardware events...");

    // Main loop: scan USB devices
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        match UdevMonitor::scan_usb_devices() {
            Ok(events) => {
                for event in events {
                    tracing::debug!(
                        "USB device: {}:{} ({})",
                        event.vendor_id,
                        event.product_id,
                        event.action
                    );

                    // Look up driver
                    if let Ok(Some(driver)) = db.lookup(&event.vendor_id, &event.product_id) {
                        tracing::info!(
                            "Found driver for {}: {} ({})",
                            event.vendor_id,
                            event.product_id,
                            driver.name
                        );

                        // Install driver
                        if let Err(e) = drivers::install_driver(&driver) {
                            tracing::warn!("Failed to install driver {}: {}", driver.name, e);
                        } else {
                            tracing::info!("Driver {} installed successfully", driver.name);

                            // Send validated notification (no jargon, max 3 buttons)
                            let _ = notifier
                                .send(Notification::simple(
                                    "Driver installed",
                                    &format!("{} is now active", driver.name),
                                ))
                                .await;
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to scan USB devices: {}", e);
            }
        }
    }
}

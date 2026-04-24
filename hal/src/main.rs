//! Hope OS Hardware Adaptation Layer (HAL+)
//!
//! Daemon that handles hardware detection, driver management,
//! and system notifications for Hope OS.

#![warn(missing_docs)]

mod db;
mod detection;
mod drivers;
mod notifications;

use log::{error, info};
use std::process;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    info!("Hope HAL+ v{} starting up", env!("CARGO_PKG_VERSION"));
    info!("Hardware Adaptation Layer for Hope OS");

    // Initialize driver database
    let driver_db = db::DriverDatabase::new()?;
    info!("Driver database initialized: {} entries", driver_db.len());

    // Initialize udev monitor
    let mut monitor = detection::UdevMonitor::new()?;
    info!("udev monitor initialized");

    // Spawn notification system
    let notification_tx = notifications::spawn_notification_system();
    info!("Notification system ready");

    // Main event loop
    loop {
        tokio::select! {
            Some(event) = monitor.next_event() => {
                handle_udev_event(event, &driver_db, &notification_tx).await;
            }
            _ = signal::ctrl_c() => {
                info!("Shutting down HAL+");
                break;
            }
        }
    }

    Ok(())
}

/// Handle a udev event and drive the detection flow
async fn handle_udev_event(
    event: detection::UdevEvent,
    driver_db: &db::DriverDatabase,
    notification_tx: &notifications::NotificationSender,
) {
    info!("udev event: action={} subsystem={} vendor:product={}:{}",
          event.action, event.subsystem, event.vendor_id, event.product_id);

    // Only process device add/change events
    if event.action != "add" && event.action != "change" {
        return;
    }

    // Look up vendor:product in local database
    match driver_db.find_driver(&event.vendor_id, &event.product_id) {
        Ok(Some(driver_info)) => {
            info!("Found driver: {} for {}:{}",
                  driver_info.name, event.vendor_id, event.product_id);

            // Install the driver
            if let Err(e) = drivers::install_driver(&driver_info) {
                error!("Failed to install driver: {}", e);
                notification_tx
                    .send_notification(notifications::Notification {
                        title: "Problème matériel".into(),
                        body: format!("L'appareil {} n'a pas de pilote disponible.", driver_info.name),
                        actions: vec![
                            notifications::Action {
                                label: "Ignorer".into(),
                                id: "ignore".into(),
                            },
                            notifications::Action {
                                label: "Plus d'infos".into(),
                                id: "info".into(),
                            },
                        ],
                    })
                    .await;
                return;
            }

            notification_tx
                .send_notification(notifications::Notification {
                    title: format!("{} prêt", driver_info.name),
                    body: "L'appareil est configuré.".into(),
                    actions: vec![
                        notifications::Action {
                            label: "OK".into(),
                            id: "ok".into(),
                        },
                    ],
                })
                .await;
        }
        Ok(None) => {
            // Unknown device - signal Hope-Mind to search
            info!("Unknown device {}:{}, requesting cloud lookup",
                  event.vendor_id, event.product_id);

            if let Err(e) = detection::request_cloud_lookup(&event.vendor_id, &event.product_id).await {
                error!("Cloud lookup failed: {}", e);
            }
        }
        Err(e) => {
            error!("Database error: {}", e);
        }
    }
}

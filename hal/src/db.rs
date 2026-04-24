//! Driver database management for Hope HAL+
//!
//! SQLite-based local database that maps vendor:product IDs
//! to driver packages and installation instructions.

use log::{error, info};
use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Database not initialized")]
    NotInitialized,
}

/// Information about a known driver
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverInfo {
    /// Human-readable device name
    pub name: String,
    /// USB vendor ID (hex without 0x)
    pub vendor_id: String,
    /// USB product ID (hex without 0x)
    pub product_id: String,
    /// Linux kernel module to load (if applicable)
    pub kernel_module: Option<String>,
    /// Package name to install
    pub package: Option<String>,
    /// Whether the device needs firmware
    pub needs_firmware: bool,
    /// Configuration snippet or instructions
    pub config: Option<String>,
}

/// Driver database wrapper
pub struct DriverDatabase {
    conn: Connection,
}

impl DriverDatabase {
    /// Open or create the driver database
    pub fn new() -> Result<Self, DbError> {
        let db_path = Self::db_path()?;
        let conn = Connection::open(&db_path)?;

        let db = Self { conn };
        db.init_schema()?;
        db.preload_builtin_drivers()?;

        Ok(db)
    }

    /// Get the database path
    fn db_path() -> Result<PathBuf, DbError> {
        let base = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let mut p = dirs::data_dir().unwrap_or_else(|| PathBuf::from("/usr/share"));
                p.push("hope");
                p
            });

        std::fs::create_dir_all(&base)?;
        Ok(base.join("hope-driver-db.sqlite"))
    }

    /// Initialize the database schema
    fn init_schema(&self) -> Result<(), DbError> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS drivers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                vendor_id TEXT NOT NULL,
                product_id TEXT NOT NULL,
                kernel_module TEXT,
                package TEXT,
                needs_firmware INTEGER DEFAULT 0,
                config TEXT,
                source TEXT DEFAULT 'builtin',
                last_seen INTEGER,
                UNIQUE(vendor_id, product_id)
            );

            CREATE INDEX IF NOT EXISTS idx_vendor_product
            ON drivers(vendor_id, product_id);

            CREATE TABLE IF NOT EXISTS sync_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                synced_at INTEGER NOT NULL,
                entries_updated INTEGER DEFAULT 0,
                status TEXT
            );
            "#,
        )?;
        Ok(())
    }

    /// Preload builtin drivers for common hardware
    fn preload_builtin_drivers(&self) -> Result<(), DbError> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM drivers", [], |r| r.get(0))?;

        if count > 0 {
            return Ok(());
        }

        info!("Preloading builtin driver database");

        let builtin_drivers = vec![
            // Intel graphics
            ("Intel Graphics", "8086", "0000", Some("i915"), None, true, None),
            // Realtek card readers
            ("Realtek Card Reader", "0bda", "0129", Some("rtsx_usb"), None, false, None),
            ("Realtek Card Reader", "0bda", "0139", Some("rtsx_usb"), None, false, None),
            // Intel wireless
            ("Intel Wi-Fi", "8087", "0a2a", Some("iwlwifi"), Some("firmware-iwlwifi"), true, None),
            ("Intel Wi-Fi", "8087", "0a2b", Some("iwlwifi"), Some("firmware-iwlwifi"), true, None),
            // Broadcom wireless
            ("Broadcom Wi-Fi", "1054", "0400", Some("brcmfmac"), Some("firmware-brcm80211"), true, None),
            // MediaTek wireless
            ("MediaTek Wi-Fi", "048d", "1236", Some("mt76"), Some("firmware-mt76"), true, None),
            // USB storage
            ("USB Storage", "090c", "1000", None, None, false, None),
            ("USB Storage", "0bda", "0151", None, None, false, None),
            ("USB Storage", "0bda", "0311", None, None, false, None),
            // Bluetooth
            ("Bluetooth", "0a5c", "216f", Some("btusb"), Some("firmware-atheros"), true, None),
            ("Bluetooth", "0b05", "17cb", Some("btusb"), Some("firmware-atheros"), true, None),
            // ThinkPad hardware
            ("ThinkPad TrackPoint", "06cb", "0001", None, None, false, None),
            ("ThinkPad Fingerprint", "06cb", "76ad", Some("serio"), None, false, None),
        ];

        for (name, vid, pid, module, package, firmware, config) in builtin_drivers {
            self.insert_driver(&DriverInfo {
                name: name.into(),
                vendor_id: vid.into(),
                product_id: pid.into(),
                kernel_module: module.map(String::from),
                package: package.map(String::from),
                needs_firmware: firmware,
                config: config.map(String::from),
            })?;
        }

        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM drivers", [], |r| r.get(0))?;

        info!("Loaded {} builtin drivers", count);

        Ok(())
    }

    /// Insert or update a driver
    pub fn insert_driver(&self, driver: &DriverInfo) -> Result<(), DbError> {
        self.conn.execute(
            r#"
            INSERT INTO drivers (name, vendor_id, product_id, kernel_module, package, needs_firmware, config)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(vendor_id, product_id) DO UPDATE SET
                name = excluded.name,
                kernel_module = excluded.kernel_module,
                package = excluded.package,
                needs_firmware = excluded.needs_firmware,
                config = excluded.config
            "#,
            params![
                driver.name,
                driver.vendor_id,
                driver.product_id,
                driver.kernel_module,
                driver.package,
                driver.needs_firmware as i32,
                driver.config,
            ],
        )?;
        Ok(())
    }

    /// Find a driver by vendor and product ID
    pub fn find_driver(&self, vendor_id: &str, product_id: &str) -> Result<Option<DriverInfo>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT name, vendor_id, product_id, kernel_module, package, needs_firmware, config
             FROM drivers WHERE vendor_id = ?1 AND product_id = ?2",
        )?;

        let result = stmt.query_row(params![vendor_id, product_id], |row| {
            Ok(DriverInfo {
                name: row.get(0)?,
                vendor_id: row.get(1)?,
                product_id: row.get(2)?,
                kernel_module: row.get(3)?,
                package: row.get(4)?,
                needs_firmware: row.get::<_, i32>(5)? != 0,
                config: row.get(6)?,
            })
        });

        match result {
            Ok(info) => Ok(Some(info)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::Sqlite(e)),
        }
    }

    /// Return the number of drivers in the database
    pub fn len(&self) -> usize {
        self.conn
            .query_row("SELECT COUNT(*)", [], |r| r.get::<_, i64>(0))
            .map(|c| c as usize)
            .unwrap_or(0)
    }

    /// Sync from cloud (placeholder for future cloud sync)
    pub async fn sync_cloud(&mut self) -> Result<(), DbError> {
        // TODO: Implement cloud sync
        info!("Cloud sync requested (not yet implemented)");
        Ok(())
    }
}

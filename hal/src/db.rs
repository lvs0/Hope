//! Driver database management for Hope HAL+

use rusqlite::{params, Connection};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverInfo {
    pub name: String,
    pub vendor_id: String,
    pub product_id: String,
    pub kernel_module: Option<String>,
    pub package: Option<String>,
    pub needs_firmware: bool,
    pub config: Option<String>,
}

pub struct DriverDatabase {
    conn: Connection,
}

impl DriverDatabase {
    pub fn new() -> Result<Self, DbError> {
        let db_path = Self::db_path()?;
        let conn = Connection::open(&db_path)?;
        let db = Self { conn };
        db.init_schema()?;
        db.preload_builtin_drivers()?;
        Ok(db)
    }

    fn db_path() -> Result<PathBuf, DbError> {
        let base = dirs::data_local_dir().ok_or(DbError::NotInitialized)?;
        let hope_dir = base.join("hope-hal");
        std::fs::create_dir_all(&hope_dir)?;
        Ok(hope_dir.join("drivers.db"))
    }

    fn init_schema(&self) -> Result<(), DbError> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS drivers (id INTEGER PRIMARY KEY, name TEXT NOT NULL, vendor_id TEXT NOT NULL, product_id TEXT NOT NULL, kernel_module TEXT, package TEXT, needs_firmware INTEGER NOT NULL, config TEXT); CREATE INDEX IF NOT EXISTS idx_vp ON drivers(vendor_id, product_id);"
        )?;
        Ok(())
    }

    fn preload_builtin_drivers(&self) -> Result<(), DbError> {
        let count: i64 = self.conn.query_row("SELECT COUNT(*) FROM drivers", [], |r| r.get(0))?;
        if count > 0 { return Ok(()); }

        let drivers = [
            ("Intel Graphics", "8086", "0000", "i915", true),
            ("Realtek Card Reader", "0bda", "0129", "rtsx_usb", false),
            ("Realtek Card Reader", "0bda", "0139", "rtsx_usb", false),
            ("Intel Wi-Fi", "8087", "0a2a", "iwlwifi", true),
            ("Intel Wi-Fi", "8087", "0a2b", "iwlwifi", true),
            ("Broadcom Wi-Fi", "1054", "0400", "brcmfmac", true),
            ("MediaTek Wi-Fi", "048d", "1236", "mt76", true),
            ("USB Storage", "090c", "1000", "", false),
            ("USB Storage", "0bda", "0151", "", false),
            ("USB Storage", "0bda", "0311", "", false),
            ("Bluetooth", "0a5c", "216f", "btusb", true),
            ("Bluetooth", "0b05", "17cb", "btusb", true),
            ("ThinkPad TrackPoint", "06cb", "0001", "", false),
            ("ThinkPad Fingerprint", "06cb", "76ad", "serio", false),
        ];

        for (name, vid, pid, km, fw) in drivers {
            let km: Option<String> = if km.is_empty() { None } else { Some(km.to_string()) };
            self.conn.execute(
                "INSERT OR IGNORE INTO drivers (name, vendor_id, product_id, kernel_module, needs_firmware, config) VALUES (?1, ?2, ?3, ?4, ?5, NULL)",
                params![name, vid, pid, km, fw],
            )?;
        }
        Ok(())
    }

    pub fn lookup(&self, vendor_id: &str, product_id: &str) -> Result<Option<DriverInfo>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT name, vendor_id, product_id, kernel_module, package, needs_firmware, config FROM drivers WHERE vendor_id = ?1 AND product_id = ?2")?;
        
        let result = stmt.query_row(params![vendor_id, product_id], |r| {
            Ok(DriverInfo {
                name: r.get(0)?,
                vendor_id: r.get(1)?,
                product_id: r.get(2)?,
                kernel_module: r.get(3)?,
                package: r.get(4)?,
                needs_firmware: r.get(5)?,
                config: r.get(6)?,
            })
        });

        match result {
            Ok(info) => Ok(Some(info)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn upsert(&self, info: &DriverInfo) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO drivers (name, vendor_id, product_id, kernel_module, package, needs_firmware, config) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![info.name, info.vendor_id, info.product_id, info.kernel_module, info.package, info.needs_firmware, info.config],
        )?;
        Ok(())
    }
}

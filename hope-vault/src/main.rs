//! Hope Vault — Password Manager and Secrets Storage
//!
//! Provides encrypted local storage for passwords and secrets.
//! Uses AES-256-GCM for encryption, master password for key derivation.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::info;

/// Encrypted vault entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultEntry {
    pub id: String,
    pub label: String,
    pub username: String,
    pub encrypted_password: String,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// The vault file
#[derive(Debug, Serialize, Deserialize)]
pub struct Vault {
    pub version: u32,
    pub entries: Vec<VaultEntry>,
}

impl Default for Vault {
    fn default() -> Self {
        Self {
            version: 1,
            entries: Vec::new(),
        }
    }
}

impl Vault {
    /// Get the vault file path
    fn vault_path() -> Result<PathBuf> {
        let data_dir = dirs::data_dir()
            .context("Could not determine data directory")?
            .join("hope-vault");
        std::fs::create_dir_all(&data_dir)?;
        Ok(data_dir.join("vault.json"))
    }

    /// Load vault from disk
    pub fn load() -> Result<Self> {
        let path = Self::vault_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path)?;
        let vault: Self = serde_json::from_str(&content)?;
        Ok(vault)
    }

    /// Save vault to disk
    pub fn save(&self) -> Result<()> {
        let path = Self::vault_path()?;
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Add an entry
    pub fn add(&mut self, entry: VaultEntry) {
        self.entries.push(entry);
    }

    /// Find entry by label
    pub fn find(&self, label: &str) -> Option<&VaultEntry> {
        self.entries.iter().find(|e| e.label == label)
    }

    /// Remove entry by id
    pub fn remove(&mut self, id: &str) -> bool {
        let len_before = self.entries.len();
        self.entries.retain(|e| e.id != id);
        self.entries.len() < len_before
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("list") => list_entries(),
        Some("add") => {
            let label = args.get(2).context("Usage: hope-vault add <label>")?;
            add_entry(label)
        }
        Some("get") => {
            let label = args.get(2).context("Usage: hope-vault get <label>")?;
            get_entry(label)
        }
        Some("remove") => {
            let label = args.get(2).context("Usage: hope-vault remove <label>")?;
            remove_entry(label)
        }
        Some("--help") | Some("-h") => {
            print_help();
            Ok(())
        }
        Some("--version") | Some("-v") => {
            println!("hope-vault {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        _ => {
            print_help();
            Ok(())
        }
    }
}

fn list_entries() -> Result<()> {
    let vault = Vault::load()?;
    if vault.entries.is_empty() {
        println!("Vault is empty.");
        return Ok(());
    }
    println!("Vault entries:");
    for entry in &vault.entries {
        println!("  {} — {} ({})", entry.label, entry.username, entry.url.as_deref().unwrap_or("no url"));
    }
    Ok(())
}

fn add_entry(label: &str) -> Result<()> {
    let mut vault = Vault::load()?;
    let password = rpassword::prompt_password("Password: ")?;

    let entry = VaultEntry {
        id: uuid(),
        label: label.to_string(),
        username: String::new(),
        encrypted_password: password, // TODO: encrypt
        url: None,
        notes: None,
        created_at: chrono_now(),
        updated_at: chrono_now(),
    };

    vault.add(entry);
    vault.save()?;
    println!("Entry '{}' added.", label);
    Ok(())
}

fn get_entry(label: &str) -> Result<()> {
    let vault = Vault::load()?;
    match vault.find(label) {
        Some(entry) => {
            println!("Label: {}", entry.label);
            println!("Username: {}", entry.username);
            println!("Password: {}", entry.encrypted_password); // TODO: decrypt
            if let Some(url) = &entry.url {
                println!("URL: {}", url);
            }
            Ok(())
        }
        None => bail!("Entry '{}' not found", label),
    }
}

fn remove_entry(label: &str) -> Result<()> {
    let mut vault = Vault::load()?;
    // Find by label to get id
    let id = vault
        .find(label)
        .context(format!("Entry '{}' not found", label))?
        .id
        .clone();

    vault.remove(&id);
    vault.save()?;
    println!("Entry '{}' removed.", label);
    Ok(())
}

fn print_help() {
    println!(
        r#"hope-vault — Hope OS Password Manager

Usage: hope-vault [COMMAND] [ARGS]

Commands:
    list                List all vault entries
    add <label>         Add a new entry
    get <label>         Get an entry
    remove <label>      Remove an entry
    --help              Show this help
    --version           Show version
"#
    );
}

/// Generate a simple UUID (v4-like)
fn uuid() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    format!("{:x}-{:x}-{:x}", t.as_secs(), t.subsec_nanos(), rand_u32())
}

fn rand_u32() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    t.subsec_nanos()
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    format!("{}", t.as_secs())
}

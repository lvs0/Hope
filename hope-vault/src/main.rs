//! Hope Vault — Password Manager and Secrets Storage
//!
//! Provides encrypted local storage for passwords and secrets.
//! Uses AES-256-GCM for encryption, PBKDF2-SHA256 for key derivation.

use anyhow::{bail, Context, Result};
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::pbkdf2;
use ring::pbkdf2::PBKDF2_HMAC_SHA256;
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;
use std::path::PathBuf;

const PBKDF2_ITERATIONS: u32 = 600_000;
const SALT_LEN: usize = 32;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

/// Encrypted vault entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultEntry {
    pub id: String,
    pub label: String,
    pub username: String,
    pub encrypted_password: String,
    pub nonce: String,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// The vault file
#[derive(Debug, Serialize, Deserialize)]
pub struct Vault {
    pub version: u32,
    pub salt: String,
    pub entries: Vec<VaultEntry>,
}

impl Default for Vault {
    fn default() -> Self {
        Self {
            version: 1,
            salt: String::new(),
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

/// Derive an AES-256 key from master password and salt using PBKDF2-SHA256
fn derive_key(password: &[u8], salt: &[u8]) -> [u8; KEY_LEN] {
    let mut key = [0u8; KEY_LEN];
    let iterations = NonZeroU32::new(PBKDF2_ITERATIONS).expect("PBKDF2_ITERATIONS must be > 0");
    pbkdf2::derive(PBKDF2_HMAC_SHA256, iterations, salt, password, &mut key);
    key
}

/// Generate cryptographically random bytes
fn random_bytes(len: usize) -> Result<Vec<u8>> {
    let rng = SystemRandom::new();
    let mut bytes = vec![0u8; len];
    rng.fill(&mut bytes)
        .map_err(|_| anyhow::anyhow!("Failed to generate random bytes"))?;
    Ok(bytes)
}

/// Encrypt plaintext with AES-256-GCM using derived key
fn encrypt(plaintext: &[u8], key: &[u8; KEY_LEN]) -> Result<(Vec<u8>, Vec<u8>)> {
    let unbound_key =
        UnboundKey::new(&AES_256_GCM, key).map_err(|_| anyhow::anyhow!("Invalid key length"))?;
    let nonce_bytes = random_bytes(NONCE_LEN)?;
    let nonce = Nonce::try_assume_unique_for_key(&nonce_bytes)
        .map_err(|_| anyhow::anyhow!("Invalid nonce length"))?;
    let mut in_out = plaintext.to_vec();
    let sealing_key = LessSafeKey::new(unbound_key);
    sealing_key
        .seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)
        .map_err(|_| anyhow::anyhow!("Encryption failed"))?;
    Ok((in_out, nonce_bytes))
}

/// Decrypt ciphertext with AES-256-GCM using derived key
fn decrypt(ciphertext: &[u8], nonce: &[u8], key: &[u8; KEY_LEN]) -> Result<Vec<u8>> {
    let unbound_key =
        UnboundKey::new(&AES_256_GCM, key).map_err(|_| anyhow::anyhow!("Invalid key length"))?;
    let nonce = Nonce::try_assume_unique_for_key(nonce)
        .map_err(|_| anyhow::anyhow!("Invalid nonce length"))?;
    let mut in_out = ciphertext.to_vec();
    let opening_key = LessSafeKey::new(unbound_key);
    let plaintext = opening_key
        .open_in_place(nonce, Aad::empty(), &mut in_out)
        .map_err(|_| anyhow::anyhow!("Decryption failed — wrong master password?"))?;
    Ok(plaintext.to_vec())
}

/// Encode bytes as hex string
fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Decode hex string to bytes
fn from_hex(hex: &str) -> Result<Vec<u8>> {
    if hex.len() % 2 != 0 {
        bail!("Invalid hex string length");
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).context("Invalid hex"))
        .collect()
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
        println!(
            "  {} — {} ({})",
            entry.label,
            entry.username,
            entry.url.as_deref().unwrap_or("no url")
        );
    }
    Ok(())
}

fn add_entry(label: &str) -> Result<()> {
    let mut vault = Vault::load()?;
    let password = rpassword::prompt_password("Password: ")?;

    // Generate salt if first entry
    let salt = if vault.salt.is_empty() {
        let s = random_bytes(SALT_LEN)?;
        vault.salt = to_hex(&s);
        s
    } else {
        from_hex(&vault.salt)?
    };

    // Derive key from master password
    let master = rpassword::prompt_password("Master password (for encryption): ")?;
    let key = derive_key(master.as_bytes(), &salt);

    // Encrypt the password
    let (encrypted, nonce) = encrypt(password.as_bytes(), &key)?;

    let entry = VaultEntry {
        id: uuid(),
        label: label.to_string(),
        username: String::new(),
        encrypted_password: to_hex(&encrypted),
        nonce: to_hex(&nonce),
        url: None,
        notes: None,
        created_at: chrono_now(),
        updated_at: chrono_now(),
    };

    vault.add(entry);
    vault.save()?;
    println!("Entry '{}' added (encrypted).", label);
    Ok(())
}

fn get_entry(label: &str) -> Result<()> {
    let vault = Vault::load()?;
    let entry = match vault.find(label) {
        Some(e) => e,
        None => bail!("Entry '{}' not found", label),
    };

    let salt = from_hex(&vault.salt)?;
    let master = rpassword::prompt_password("Master password: ")?;
    let key = derive_key(master.as_bytes(), &salt);

    let ciphertext = from_hex(&entry.encrypted_password)?;
    let nonce = from_hex(&entry.nonce)?;
    let plaintext = decrypt(&ciphertext, &nonce, &key)?;

    let password =
        String::from_utf8(plaintext).context("Decrypted password is not valid UTF-8")?;

    println!("Label: {}", entry.label);
    println!("Username: {}", entry.username);
    println!("Password: {}", password);
    if let Some(url) = &entry.url {
        println!("URL: {}", url);
    }
    Ok(())
}

fn remove_entry(label: &str) -> Result<()> {
    let mut vault = Vault::load()?;
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
    add <label>         Add a new entry (prompts for password + master password)
    get <label>         Decrypt and show an entry (prompts for master password)
    remove <label>      Remove an entry
    --help              Show this help
    --version           Show version

Security:
    AES-256-GCM encryption with PBKDF2-SHA256 key derivation (600k iterations).
    Master password never stored — required for encrypt/decrypt.
"#
    );
}

/// Generate a UUID v4-like string using cryptographic randomness
fn uuid() -> String {
    let bytes = random_bytes(16).unwrap_or_else(|_| {
        // Fallback to timestamp if RNG fails (should not happen)
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        vec![
            (t.as_secs() >> 24) as u8,
            (t.as_secs() >> 16) as u8,
            (t.as_secs() >> 8) as u8,
            t.as_secs() as u8,
            t.subsec_nanos() as u8,
            (t.subsec_nanos() >> 8) as u8,
            (t.subsec_nanos() >> 16) as u8,
            (t.subsec_nanos() >> 24) as u8,
            0, 0, 0, 0, 0, 0, 0, 0,
        ]
    });

    // Set version nibble (4) and variant bits (10xx)
    let mut b = bytes;
    b[6] = (b[6] & 0x0f) | 0x40;
    b[8] = (b[8] & 0x3f) | 0x80;

    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        b[0], b[1], b[2], b[3],
        b[4], b[5],
        b[6], b[7],
        b[8], b[9],
        b[10], b[11], b[12], b[13], b[14], b[15],
    )
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap();
    format!("{}", t.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = derive_key(b"test-password", b"test-salt-32-bytes-long!!!!!!!");
        let plaintext = b"my-secret-password-123";
        let (encrypted, nonce) = encrypt(plaintext, &key).unwrap();
        let decrypted = decrypt(&encrypted, &nonce, &key).unwrap();
        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_encrypt_wrong_key_fails() {
        let key1 = derive_key(b"password1", b"salt");
        let key2 = derive_key(b"password2", b"salt");
        let plaintext = b"secret";
        let (encrypted, nonce) = encrypt(plaintext, &key1).unwrap();
        assert!(decrypt(&encrypted, &nonce, &key2).is_err());
    }

    #[test]
    fn test_hex_roundtrip() {
        let data = vec![0u8, 1, 2, 127, 128, 255];
        let hex = to_hex(&data);
        let decoded = from_hex(&hex).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn test_uuid_format() {
        let id = uuid();
        assert_eq!(id.len(), 36);
        assert_eq!(id.chars().filter(|c| *c == '-').count(), 4);
    }
}

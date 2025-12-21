use std::path::{Path, PathBuf};
use std::fs;
use anyhow::Result;
use iroh::SecretKey;
use uuid::Uuid;

use crate::core::{crypto, device};

const FOOTNOTES_DIR: &str = ".footnotes";
const CONTACTS_DIR: &str = "contacts";
const TRUSTED_SOURCES_DIR: &str = "footnotes";
const LOCAL_DEVICE_KEY_FILE: &str = "this_device";
const MASTER_KEY_FILE: &str = "master_identity";

/// Represents a footnote vault at a specific path
pub struct Vault {
    path: PathBuf,
}

impl Vault {
    /// Check if a path contains a valid vault
    pub fn is_valid(path: &Path) -> bool {
        path.join(FOOTNOTES_DIR).exists()
    }

    /// Create a new vault at the given path with initial device
    pub fn create(path: PathBuf, username: &str, device_name: &str) -> Result<Self> {
        // Check if already initialized
        if Self::is_valid(&path) {
            anyhow::bail!(
                "Vault already exists at {}. Remove it first if you want to reinitialize.",
                path.display()
            );
        }

        eprintln!("Initializing vault at {}", path.display());

        // Create directory structure
        let footnotes_dir = path.join(FOOTNOTES_DIR);
        let contacts_dir = footnotes_dir.join(CONTACTS_DIR);
        let trusted_sources_dir = path.join(TRUSTED_SOURCES_DIR);

        fs::create_dir_all(&footnotes_dir)?;
        fs::create_dir_all(&contacts_dir)?;
        fs::create_dir_all(&trusted_sources_dir)?;

        // Generate master identity key pair
        eprintln!("Generating master identity key...");
        let (signing_key, verifying_key) = crypto::generate_identity_keypair();

        // Store master private key
        let master_key_file = footnotes_dir.join(MASTER_KEY_FILE);
        fs::write(&master_key_file, crypto::signing_key_to_hex(&signing_key))?;
        eprintln!(
            "Master identity key stored at {}",
            master_key_file.display()
        );

        // Generate Iroh endpoint for this device
        eprintln!("Generating Iroh endpoint for this device...");
        let secret_key = SecretKey::generate(&mut rand::rng());
        let public_key = secret_key.public();

        // Store Iroh secret key
        let key_file = footnotes_dir.join(LOCAL_DEVICE_KEY_FILE);
        fs::write(&key_file, secret_key.to_bytes())?;
        eprintln!("Device Iroh key stored at {}", key_file.display());

        // Create device-specific home note at vault root
        let home_uuid = Uuid::new_v4();
        let home_filename = format!("home-{}.md", device_name);
        let home_file = path.join(&home_filename);
        let home_content = format!(
            r#"---
uuid: {}
share_with: []
---

# Home ({})

Welcome to footnote! This is your home note.
"#,
            home_uuid, device_name
        );
        fs::write(&home_file, home_content)?;
        eprintln!("Home note created at {}", home_file.display());

        // Create contact.json with initial device
        eprintln!("Creating contact record...");
        let contact_timestamp = chrono::Utc::now().to_rfc3339();

        let contact_device = crypto::ContactDevice {
            device_name: device_name.to_string(),
            iroh_endpoint_id: public_key.to_string(),
            added_at: contact_timestamp.clone(),
        };

        let mut contact_record = crypto::ContactRecord {
            username: username.to_string(),
            nickname: String::new(),
            master_public_key: crypto::verifying_key_to_hex(&verifying_key),
            devices: vec![contact_device],
            updated_at: contact_timestamp,
            signature: String::new(),
        };

        let signature = crypto::sign_contact_record(&contact_record, &signing_key)?;
        contact_record.signature = signature;

        let contact_path = footnotes_dir.join("contact.json");
        fs::write(
            &contact_path,
            serde_json::to_string_pretty(&contact_record)?,
        )?;
        eprintln!("Contact record created at {}", contact_path.display());

        eprintln!("\nVault initialization complete!");

        Ok(Self { path })
    }

    /// Open an existing vault at the given path
    pub fn open(path: PathBuf) -> Result<Self> {
        if !Self::is_valid(&path) {
            anyhow::bail!(
                "Not a valid vault: {} (missing .footnotes directory)",
                path.display()
            );
        }

        Ok(Self { path })
    }

    /// Get the vault root path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get the .footnotes directory path
    pub fn footnotes_dir(&self) -> PathBuf {
        self.path.join(FOOTNOTES_DIR)
    }

    /// Get the local device name for this vault
    pub fn local_device_name(&self) -> Result<String> {
        device::get_local_device_name(&self.path)
    }

    /// Get the master public key for this vault
    pub fn master_public_key(&self) -> Result<String> {
        let contact_path = self.footnotes_dir().join("contact.json");
        let contact_content = fs::read_to_string(&contact_path)?;
        let contact_record: crypto::ContactRecord = serde_json::from_str(&contact_content)?;
        Ok(contact_record.master_public_key)
    }

    /// Get the local device's endpoint ID
    pub fn device_endpoint_id(&self) -> Result<String> {
        let key_file = self.footnotes_dir().join(LOCAL_DEVICE_KEY_FILE);
        let key_bytes = fs::read(&key_file)?;
        let key_array: [u8; 32] = key_bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid key length"))?;
        let secret_key = SecretKey::from_bytes(&key_array);
        Ok(secret_key.public().to_string())
    }
}

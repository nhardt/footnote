use crate::core::crypto;
use iroh::SecretKey;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

const DEFAULT_DEVICE_NAME: &str = "primary";
const DEFAULT_USERNAME: &str = "me";
const LOCAL_DEVICE_KEY_FILE: &str = "this_device";
const MASTER_KEY_FILE: &str = "master_identity";

#[derive(Serialize)]
struct HqCreateOutput {
    vault_path: String,
    master_public_key: String,
    device_name: String,
    device_endpoint_id: String,
}

/// Create HQ (headquarters) - primary device and vault initialization
pub async fn create_hq(
    path: Option<PathBuf>,
    username: Option<&str>,
    device_name: Option<&str>,
) -> anyhow::Result<()> {
    let username = username.unwrap_or(DEFAULT_USERNAME);
    let device_name = device_name.unwrap_or(DEFAULT_DEVICE_NAME);

    // Determine vault path: use provided path or current directory
    let vault_path = match path {
        Some(p) => p,
        None => std::env::current_dir()
            .map_err(|_| anyhow::anyhow!("Failed to get current directory"))?,
    };

    // Check if already initialized
    let fieldnotes_dir = vault_path.join(".fieldnotes");
    if fieldnotes_dir.exists() {
        anyhow::bail!(
            "Vault already exists at {}. Remove it first if you want to reinitialize.",
            vault_path.display()
        );
    }

    eprintln!("Creating HQ at {}", vault_path.display());

    // Create directory structure
    let fieldnotes_dir = vault_path.join(".fieldnotes");
    let outposts_dir = vault_path.join("outposts");
    let notes_dir = vault_path.join("notes");
    let embassies_dir = vault_path.join("embassies");

    fs::create_dir_all(&fieldnotes_dir)?;
    fs::create_dir_all(&outposts_dir)?;
    fs::create_dir_all(&notes_dir)?;
    fs::create_dir_all(&embassies_dir)?;

    // Generate master identity key pair
    eprintln!("Generating master identity key...");
    let (signing_key, verifying_key) = crypto::generate_identity_keypair();

    // Store master private key
    let master_key_file = fieldnotes_dir.join(MASTER_KEY_FILE);
    fs::write(&master_key_file, crypto::signing_key_to_hex(&signing_key))?;
    eprintln!("Master identity key stored at {}", master_key_file.display());

    // Create identity.md
    let identity_file = vault_path.join("identity.md");
    let identity_content = format!(
        r#"---
master_public_key: {}
nickname: ""
---

# My Identity

This file contains your master identity information.
Edit the nickname field to set how you present yourself to others.
"#,
        crypto::verifying_key_to_hex(&verifying_key)
    );
    fs::write(&identity_file, identity_content)?;
    eprintln!("Identity file created at {}", identity_file.display());

    // Generate Iroh endpoint for this device
    eprintln!("Generating Iroh endpoint for this device...");
    let secret_key = SecretKey::generate(&mut rand::rng());
    let public_key = secret_key.public();

    // Store Iroh secret key
    let key_file = fieldnotes_dir.join(LOCAL_DEVICE_KEY_FILE);
    fs::write(&key_file, secret_key.to_bytes())?;
    eprintln!("Device Iroh key stored at {}", key_file.display());

    // Sign the device record with master key
    let timestamp = chrono::Utc::now().to_rfc3339();
    let signature = crypto::sign_device_record(
        device_name,
        &public_key.to_string(),
        &signing_key,
        &timestamp,
    )?;

    // Create device markdown file with signature
    let device_file = outposts_dir.join(format!("{}.md", device_name));
    let device_content = format!(
        r#"---
device_name: {}
iroh_endpoint_id: {}
authorized_by: {}
timestamp: {}
signature: {}
---

This is the device file for this device.
"#,
        device_name,
        public_key,
        crypto::verifying_key_to_hex(&verifying_key),
        timestamp,
        signature
    );
    fs::write(&device_file, device_content)?;
    eprintln!("Device file created and signed at {}", device_file.display());

    // Create home note
    let home_uuid = Uuid::new_v4();
    let home_file = notes_dir.join("home.md");
    let home_content = format!(
        r#"---
uuid: {}
share_with: []
---

# Home

Welcome to fieldnote! This is your home note.
"#,
        home_uuid
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

    let contact_path = vault_path.join(".fieldnotes").join("contact.json");
    fs::write(&contact_path, serde_json::to_string_pretty(&contact_record)?)?;
    eprintln!("Contact record created at {}", contact_path.display());

    eprintln!("\nHQ creation complete!");

    // Output JSON to stdout
    let output = HqCreateOutput {
        vault_path: vault_path.display().to_string(),
        master_public_key: crypto::verifying_key_to_hex(&verifying_key),
        device_name: device_name.to_string(),
        device_endpoint_id: public_key.to_string(),
    };
    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(())
}

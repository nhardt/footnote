use crate::core::{crypto, vault};
use iroh::SecretKey;
use std::fs;
use uuid::Uuid;

const DEFAULT_DEVICE_NAME: &str = "primary";
const LOCAL_DEVICE_KEY_FILE: &str = "this_device";
const MASTER_KEY_FILE: &str = "master_identity";

/// Create HQ (headquarters) - primary device and vault initialization
pub async fn create_hq(device_name: Option<&str>) -> anyhow::Result<()> {
    let device_name = device_name.unwrap_or(DEFAULT_DEVICE_NAME);
    let vault_path = vault::get_vault_path()?;

    // Check if already initialized
    if vault_path.exists() {
        anyhow::bail!(
            "Vault already exists at {}. Remove it first if you want to reinitialize.",
            vault_path.display()
        );
    }

    println!("Creating HQ at {}", vault_path.display());

    // Create directory structure
    let keys_dir = vault::get_keys_dir()?;
    let outposts_dir = vault::get_outposts_dir()?;
    let notes_dir = vault::get_notes_dir()?;
    let embassies_dir = vault::get_embassies_dir()?;

    fs::create_dir_all(&keys_dir)?;
    fs::create_dir_all(&outposts_dir)?;
    fs::create_dir_all(&notes_dir)?;
    fs::create_dir_all(&embassies_dir)?;

    // Generate master identity key pair
    println!("Generating master identity key...");
    let (signing_key, verifying_key) = crypto::generate_identity_keypair();

    // Store master private key
    let master_key_file = keys_dir.join(MASTER_KEY_FILE);
    fs::write(&master_key_file, crypto::signing_key_to_hex(&signing_key))?;
    println!("Master identity key stored at {}", master_key_file.display());

    // Create identity.md
    let identity_file = vault::get_identity_path()?;
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
    println!("Identity file created at {}", identity_file.display());

    // Generate Iroh endpoint for this device
    println!("Generating Iroh endpoint for this device...");
    let secret_key = SecretKey::generate(&mut rand::rng());
    let public_key = secret_key.public();

    // Store Iroh secret key
    let key_file = keys_dir.join(LOCAL_DEVICE_KEY_FILE);
    fs::write(&key_file, secret_key.to_bytes())?;
    println!("Device Iroh key stored at {}", key_file.display());

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
    println!("Device file created and signed at {}", device_file.display());

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
    println!("Home note created at {}", home_file.display());

    println!("\nHQ creation complete!");
    println!("Your master identity key: {}", crypto::verifying_key_to_hex(&verifying_key));
    println!("Your HQ device endpoint ID: {}", public_key);
    println!("\nNext steps:");
    println!("1. Edit {} to set your nickname", identity_file.display());
    println!("2. Run 'fieldnote user read' to view your identity");

    Ok(())
}

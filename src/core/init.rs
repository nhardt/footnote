use crate::core::vault;
use iroh::SecretKey;
use std::fs;
use uuid::Uuid;

const DEVICE_NAME: &str = "this_device";

/// Initialize the fieldnote vault
pub async fn initialize() -> anyhow::Result<()> {
    let vault_path = vault::get_vault_path()?;

    // Check if already initialized
    if vault_path.exists() {
        anyhow::bail!(
            "Vault already exists at {}. Remove it first if you want to reinitialize.",
            vault_path.display()
        );
    }

    println!("Initializing fieldnote vault at {}", vault_path.display());

    // Create directory structure
    let keys_dir = vault::get_keys_dir()?;
    let devices_dir = vault::get_devices_dir()?;
    let notes_dir = vault::get_notes_dir()?;
    let outpost_dir = vault::get_outpost_dir()?;

    fs::create_dir_all(&keys_dir)?;
    fs::create_dir_all(&devices_dir)?;
    fs::create_dir_all(&notes_dir)?;
    fs::create_dir_all(&outpost_dir)?;

    // Generate secret key for this device
    println!("Generating secret key for this device...");
    let secret_key = SecretKey::generate(&mut rand::rng());
    let public_key = secret_key.public();

    // Store secret key
    let key_file = keys_dir.join(DEVICE_NAME);
    fs::write(&key_file, secret_key.to_bytes())?;
    println!("Secret key stored at {}", key_file.display());

    // Create device markdown file
    let device_file = devices_dir.join(format!("{}.md", DEVICE_NAME));
    let device_content = format!(
        r#"---
iroh_endpoint_id: {}
---

This is the device file for this device.
"#,
        public_key
    );
    fs::write(&device_file, device_content)?;
    println!("Device file created at {}", device_file.display());

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

    println!("\nInitialization complete!");
    println!("Your endpoint ID: {}", public_key);

    Ok(())
}

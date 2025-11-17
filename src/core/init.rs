use iroh::SecretKey;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

const VAULT_DIR: &str = "fieldnotes-vault";
const DEVICE_NAME: &str = "this_device";

/// Get the base vault directory path
fn get_vault_path() -> anyhow::Result<PathBuf> {
    let home = std::env::var("HOME")
        .map_err(|_| anyhow::anyhow!("HOME environment variable not set"))?;
    Ok(PathBuf::from(home).join(VAULT_DIR))
}

/// Initialize the fieldnote vault
pub async fn initialize() -> anyhow::Result<()> {
    let vault_path = get_vault_path()?;

    // Check if already initialized
    if vault_path.exists() {
        anyhow::bail!(
            "Vault already exists at {}. Remove it first if you want to reinitialize.",
            vault_path.display()
        );
    }

    println!("Initializing fieldnote vault at {}", vault_path.display());

    // Create directory structure
    let keys_dir = vault_path.join(".keys");
    let me_devices_dir = vault_path.join("me/devices");
    let me_notes_dir = vault_path.join("me/notes");

    fs::create_dir_all(&keys_dir)?;
    fs::create_dir_all(&me_devices_dir)?;
    fs::create_dir_all(&me_notes_dir)?;

    // Generate secret key for this device
    println!("Generating secret key for this device...");
    let secret_key = SecretKey::generate(&mut rand::rng());
    let public_key = secret_key.public();

    // Store secret key
    let key_file = keys_dir.join(DEVICE_NAME);
    fs::write(&key_file, secret_key.to_bytes())?;
    println!("Secret key stored at {}", key_file.display());

    // Create device markdown file
    let device_file = me_devices_dir.join(format!("{}.md", DEVICE_NAME));
    let device_content = format!(
        r#"---
iroh-endpoint-id: {}
---

This is the device file for this device.
"#,
        public_key
    );
    fs::write(&device_file, device_content)?;
    println!("Device file created at {}", device_file.display());

    // Create home note
    let home_uuid = Uuid::new_v4();
    let home_file = me_notes_dir.join("home.md");
    let home_content = format!(
        r#"---
uuid: {}
share-with: []
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

use crate::core::{crypto, vault};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
struct IdentityFrontmatter {
    master_public_key: String,
    nickname: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeviceFrontmatter {
    device_name: String,
    iroh_endpoint_id: String,
    authorized_by: String,
    timestamp: String,
    signature: String,
}

#[derive(Debug, Serialize)]
struct Device {
    name: String,
    endpoint_id: String,
    authorized_by: String,
    timestamp: String,
    signature_valid: bool,
}

#[derive(Debug, Serialize)]
struct User {
    name: String,
    master_public_key: Option<String>,
    nickname: Option<String>,
    devices: Vec<Device>,
}

#[derive(Debug, Serialize)]
struct UsersOutput {
    users: Vec<User>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExportDevice {
    device_name: String,
    iroh_endpoint_id: String,
    authorized_by: String,
    timestamp: String,
    signature: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserExport {
    nickname: String,
    master_public_key: String,
    devices: Vec<ExportDevice>,
}

/// Create a new user
pub async fn create(user_name: &str) -> anyhow::Result<()> {
    println!("TODO: user::create({})", user_name);
    Ok(())
}

/// Delete a user
pub async fn delete(user_name: &str) -> anyhow::Result<()> {
    println!("TODO: user::delete({})", user_name);
    Ok(())
}

/// Export a user's contact information
pub async fn export(user_name: &str) -> anyhow::Result<()> {
    if user_name == "me" {
        return export_me().await;
    } else {
        return export_embassy(user_name).await;
    }
}

/// Export "me" user's contact information
async fn export_me() -> anyhow::Result<()> {
    let contact_path = vault::get_contact_path()?;

    if !contact_path.exists() {
        anyhow::bail!("Contact record not found. Run 'fieldnote hq create' first.");
    }

    let content = fs::read_to_string(&contact_path)?;
    let contact_record: crypto::ContactRecord = serde_json::from_str(&content)?;

    if !crypto::verify_contact_signature(&contact_record)? {
        anyhow::bail!("Contact record signature verification failed");
    }

    println!("{}", serde_json::to_string_pretty(&contact_record)?);

    Ok(())
}

/// Export an embassy user's contact information
async fn export_embassy(user_name: &str) -> anyhow::Result<()> {
    let contact_path = vault::get_embassy_contact_path(user_name)?;

    if !contact_path.exists() {
        anyhow::bail!("Embassy '{}' not found", user_name);
    }

    let content = fs::read_to_string(&contact_path)?;
    let contact_record: crypto::ContactRecord = serde_json::from_str(&content)?;

    if !crypto::verify_contact_signature(&contact_record)? {
        anyhow::bail!("Embassy '{}' contact signature verification failed", user_name);
    }

    println!("{}", serde_json::to_string_pretty(&contact_record)?);

    Ok(())
}

/// Import a user's contact information
pub async fn import(file_path: &str, petname: &str) -> anyhow::Result<()> {
    let content = fs::read_to_string(file_path)?;
    let new_contact: crypto::ContactRecord = serde_json::from_str(&content)?;

    if !crypto::verify_contact_signature(&new_contact)? {
        anyhow::bail!("Contact signature verification failed. Import aborted.");
    }

    eprintln!(
        "Contact signature verified ({} devices)",
        new_contact.devices.len()
    );

    let contact_path = vault::get_embassy_contact_path(petname)?;

    if contact_path.exists() {
        let existing_content = fs::read_to_string(&contact_path)?;
        let existing_contact: crypto::ContactRecord = serde_json::from_str(&existing_content)?;

        let new_timestamp = chrono::DateTime::parse_from_rfc3339(&new_contact.updated_at)?;
        let existing_timestamp = chrono::DateTime::parse_from_rfc3339(&existing_contact.updated_at)?;

        if new_timestamp <= existing_timestamp {
            anyhow::bail!(
                "Embassy '{}' already has a more recent contact record (existing: {}, new: {}). Skipping update.",
                petname,
                existing_contact.updated_at,
                new_contact.updated_at
            );
        }

        eprintln!(
            "Updating embassy '{}' (old: {}, new: {})",
            petname,
            existing_contact.updated_at,
            new_contact.updated_at
        );
    }

    let embassies_dir = vault::get_embassies_dir()?;
    fs::create_dir_all(&embassies_dir)?;

    fs::write(&contact_path, serde_json::to_string_pretty(&new_contact)?)?;
    eprintln!("Contact saved to {}", contact_path.display());

    eprintln!("\nImport complete!");
    eprintln!("Embassy '{}' contact updated", petname);

    Ok(())
}

/// Read and display all users and their devices
pub async fn read() -> anyhow::Result<()> {
    let mut users = Vec::new();

    // First, add "me" user from root-level identity.md and devices/
    let identity_path = vault::get_identity_path()?;
    let (master_public_key, nickname) = if identity_path.exists() {
        let content = fs::read_to_string(&identity_path)?;
        let identity = parse_identity_frontmatter(&content);
        identity
    } else {
        (None, None)
    };

    let outposts_dir = vault::get_outposts_dir()?;
    let mut me_devices = Vec::new();

    if outposts_dir.exists() {
        for device_entry in fs::read_dir(&outposts_dir)? {
            let device_entry = device_entry?;
            let device_path = device_entry.path();

            if device_path.extension().and_then(|s| s.to_str()) == Some("md") {
                let content = fs::read_to_string(&device_path)?;
                if let Some(device) = parse_device_frontmatter(&content) {
                    me_devices.push(device);
                }
            }
        }
    }

    users.push(User {
        name: "me".to_string(),
        master_public_key,
        nickname,
        devices: me_devices,
    });

    // Scan embassies/ directory for embassy info files
    let embassies_dir = vault::get_embassies_dir()?;
    if embassies_dir.exists() {
        for entry in fs::read_dir(&embassies_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Look for *_info.md files
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                let filename = path.file_stem().unwrap().to_string_lossy();
                if filename.ends_with("_info") {
                    // Extract user name from filename (remove _info suffix)
                    let user_name = filename.strip_suffix("_info").unwrap().to_string();

                    // Read and parse the info file
                    let content = fs::read_to_string(&path)?;
                    let frontmatter = extract_frontmatter(&content);

                    if let Some(fm) = frontmatter {
                        // Parse the UserExport struct from frontmatter
                        if let Ok(export) = serde_yaml::from_str::<UserExport>(&fm) {
                            // Convert devices to Device struct
                            let mut devices = Vec::new();
                            for device in &export.devices {
                                let signature_valid = crypto::verify_device_signature(
                                    &device.device_name,
                                    &device.iroh_endpoint_id,
                                    &device.authorized_by,
                                    &device.timestamp,
                                    &device.signature,
                                )
                                .unwrap_or(false);

                                devices.push(Device {
                                    name: device.device_name.clone(),
                                    endpoint_id: device.iroh_endpoint_id.clone(),
                                    authorized_by: device.authorized_by.clone(),
                                    timestamp: device.timestamp.clone(),
                                    signature_valid,
                                });
                            }

                            users.push(User {
                                name: user_name,
                                master_public_key: Some(export.master_public_key),
                                nickname: Some(export.nickname),
                                devices,
                            });
                        }
                    }
                }
            }
        }
    }

    // Serialize and print as YAML
    let output = UsersOutput { users };
    let yaml = serde_yaml::to_string(&output)?;
    println!("{}", yaml);

    Ok(())
}

/// Parse identity frontmatter from a markdown file
fn parse_identity_frontmatter(content: &str) -> (Option<String>, Option<String>) {
    let frontmatter = extract_frontmatter(content);
    if frontmatter.is_none() {
        return (None, None);
    }

    match serde_yaml::from_str::<IdentityFrontmatter>(&frontmatter.unwrap()) {
        Ok(parsed) => {
            let nickname = if parsed.nickname.is_empty() {
                None
            } else {
                Some(parsed.nickname)
            };
            (Some(parsed.master_public_key), nickname)
        }
        Err(_) => (None, None),
    }
}

/// Parse device frontmatter from a markdown file and verify signature
fn parse_device_frontmatter(content: &str) -> Option<Device> {
    let frontmatter = extract_frontmatter(content)?;

    let parsed: DeviceFrontmatter = serde_yaml::from_str(&frontmatter).ok()?;

    // Verify the signature
    let signature_valid = crypto::verify_device_signature(
        &parsed.device_name,
        &parsed.iroh_endpoint_id,
        &parsed.authorized_by,
        &parsed.timestamp,
        &parsed.signature,
    )
    .unwrap_or(false);

    Some(Device {
        name: parsed.device_name,
        endpoint_id: parsed.iroh_endpoint_id,
        authorized_by: parsed.authorized_by,
        timestamp: parsed.timestamp,
        signature_valid,
    })
}

/// Extract YAML frontmatter from markdown content
fn extract_frontmatter(content: &str) -> Option<String> {
    let mut lines = content.lines();

    // Check for opening ---
    if lines.next()?.trim() != "---" {
        return None;
    }

    // Collect lines until closing ---
    let mut frontmatter_lines = Vec::new();
    for line in lines {
        if line.trim() == "---" {
            break;
        }
        frontmatter_lines.push(line);
    }

    Some(frontmatter_lines.join("\n"))
}

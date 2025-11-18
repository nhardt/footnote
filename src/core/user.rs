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
    // Determine paths based on user name
    let (identity_path, devices_dir) = if user_name == "me" {
        (vault::get_identity_path()?, vault::get_devices_dir()?)
    } else {
        (
            vault::get_user_identity_path(user_name)?,
            vault::get_user_devices_dir(user_name)?,
        )
    };

    // Read identity
    if !identity_path.exists() {
        anyhow::bail!("User '{}' not found", user_name);
    }

    let content = fs::read_to_string(&identity_path)?;
    let (master_public_key, nickname) = parse_identity_frontmatter(&content);

    let master_public_key = master_public_key
        .ok_or_else(|| anyhow::anyhow!("No master public key found for user '{}'", user_name))?;

    // Read devices
    let mut devices = Vec::new();
    if devices_dir.exists() {
        for device_entry in fs::read_dir(&devices_dir)? {
            let device_entry = device_entry?;
            let device_path = device_entry.path();

            if device_path.extension().and_then(|s| s.to_str()) == Some("md") {
                let content = fs::read_to_string(&device_path)?;
                let frontmatter = extract_frontmatter(&content);
                if let Some(fm) = frontmatter {
                    if let Ok(parsed) = serde_yaml::from_str::<DeviceFrontmatter>(&fm) {
                        devices.push(ExportDevice {
                            device_name: parsed.device_name,
                            iroh_endpoint_id: parsed.iroh_endpoint_id,
                            authorized_by: parsed.authorized_by,
                            timestamp: parsed.timestamp,
                            signature: parsed.signature,
                        });
                    }
                }
            }
        }
    }

    // Create export struct
    let export = UserExport {
        nickname: nickname.unwrap_or_default(),
        master_public_key,
        devices,
    };

    // Serialize and print
    let yaml = serde_yaml::to_string(&export)?;
    println!("{}", yaml);

    Ok(())
}

/// Import a user's contact information
pub async fn import(file_path: &str, petname: &str) -> anyhow::Result<()> {
    // Read and parse the export file
    let content = fs::read_to_string(file_path)?;
    let export: UserExport = serde_yaml::from_str(&content)?;

    // Verify all device signatures
    for device in &export.devices {
        let valid = crypto::verify_device_signature(
            &device.device_name,
            &device.iroh_endpoint_id,
            &device.authorized_by,
            &device.timestamp,
            &device.signature,
        )?;

        if !valid {
            anyhow::bail!(
                "Invalid signature for device '{}'. Import aborted.",
                device.device_name
            );
        }
    }

    println!(
        "✓ All signatures verified ({} devices)",
        export.devices.len()
    );

    // Create outpost directory for this user
    let outpost_dir = vault::get_user_outpost_dir(petname)?;
    if outpost_dir.exists() {
        anyhow::bail!(
            "User '{}' already exists in outpost. Remove it first if you want to re-import.",
            petname
        );
    }

    let identity_path = vault::get_user_identity_path(petname)?;
    let devices_dir = vault::get_user_devices_dir(petname)?;
    let notes_dir = vault::get_user_notes_dir(petname)?;

    fs::create_dir_all(&devices_dir)?;
    fs::create_dir_all(&notes_dir)?;

    // Create identity.md
    let identity_content = format!(
        r#"---
master_public_key: {}
nickname: {}
---

# {}

Imported contact.
Petname: {}
"#,
        export.master_public_key,
        export.nickname,
        export.nickname,
        petname
    );
    fs::write(&identity_path, identity_content)?;
    println!("✓ Created identity for '{}'", petname);

    // Create device files
    for device in &export.devices {
        let device_file = devices_dir.join(format!("{}.md", device.device_name));
        let device_content = format!(
            r#"---
device_name: {}
iroh_endpoint_id: {}
authorized_by: {}
timestamp: {}
signature: {}
---

Device imported from contact export.
"#,
            device.device_name,
            device.iroh_endpoint_id,
            device.authorized_by,
            device.timestamp,
            device.signature
        );
        fs::write(&device_file, device_content)?;
        println!("✓ Created device '{}'", device.device_name);
    }

    println!("\n✓ Import complete!");
    println!("User '{}' added to outpost at {}", petname, outpost_dir.display());

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

    let me_devices_dir = vault::get_devices_dir()?;
    let mut me_devices = Vec::new();

    if me_devices_dir.exists() {
        for device_entry in fs::read_dir(&me_devices_dir)? {
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

    // Scan outpost/ directory for other users
    let outpost_dir = vault::get_outpost_dir()?;
    if outpost_dir.exists() {
        for entry in fs::read_dir(&outpost_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let user_name = path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();

                // Read identity for this user
                let user_identity_path = path.join("identity.md");
                let (master_public_key, nickname) = if user_identity_path.exists() {
                    let content = fs::read_to_string(&user_identity_path)?;
                    parse_identity_frontmatter(&content)
                } else {
                    (None, None)
                };

                // Scan for devices in this user's outpost
                let devices_dir = path.join("devices");
                let mut devices = Vec::new();

                if devices_dir.exists() {
                    for device_entry in fs::read_dir(&devices_dir)? {
                        let device_entry = device_entry?;
                        let device_path = device_entry.path();

                        if device_path.extension().and_then(|s| s.to_str()) == Some("md") {
                            let content = fs::read_to_string(&device_path)?;
                            if let Some(device) = parse_device_frontmatter(&content) {
                                devices.push(device);
                            }
                        }
                    }
                }

                users.push(User {
                    name: user_name,
                    master_public_key,
                    nickname,
                    devices,
                });
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

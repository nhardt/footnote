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

/// Read and display all users and their devices
pub async fn read() -> anyhow::Result<()> {
    let vault_path = vault::get_vault_path()?;

    if !vault_path.exists() {
        anyhow::bail!(
            "Vault not found at {}. Run 'fieldnote init' first.",
            vault_path.display()
        );
    }

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

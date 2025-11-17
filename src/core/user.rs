use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const VAULT_DIR: &str = "fieldnotes-vault";

/// Get the base vault directory path
fn get_vault_path() -> anyhow::Result<PathBuf> {
    let home = std::env::var("HOME")
        .map_err(|_| anyhow::anyhow!("HOME environment variable not set"))?;
    Ok(PathBuf::from(home).join(VAULT_DIR))
}

#[derive(Debug, Serialize, Deserialize)]
struct DeviceFrontmatter {
    iroh_endpoint_id: String,
}

#[derive(Debug, Serialize)]
struct Device {
    name: String,
    endpoint_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct User {
    name: String,
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
    let vault_path = get_vault_path()?;

    if !vault_path.exists() {
        anyhow::bail!(
            "Vault not found at {}. Run 'fieldnote init' first.",
            vault_path.display()
        );
    }

    let mut users = Vec::new();

    // Scan for user directories
    for entry in fs::read_dir(&vault_path)? {
        let entry = entry?;
        let path = entry.path();

        // Skip hidden directories and files
        if let Some(name) = path.file_name() {
            let name_str = name.to_string_lossy();
            if name_str.starts_with('.') {
                continue;
            }
        }

        if path.is_dir() {
            let user_name = path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();

            // Scan for devices
            let devices_dir = path.join("devices");
            let mut devices = Vec::new();

            if devices_dir.exists() {
                for device_entry in fs::read_dir(&devices_dir)? {
                    let device_entry = device_entry?;
                    let device_path = device_entry.path();

                    if device_path.extension().and_then(|s| s.to_str()) == Some("md") {
                        let device_name = device_path
                            .file_stem()
                            .unwrap()
                            .to_string_lossy()
                            .to_string();

                        // Parse endpoint ID from frontmatter
                        let content = fs::read_to_string(&device_path)?;
                        let endpoint_id = parse_frontmatter(&content);

                        devices.push(Device {
                            name: device_name,
                            endpoint_id,
                        });
                    }
                }
            }

            users.push(User {
                name: user_name,
                devices,
            });
        }
    }

    // Serialize and print as YAML
    let output = UsersOutput { users };
    let yaml = serde_yaml::to_string(&output)?;
    println!("{}", yaml);

    Ok(())
}

/// Parse the frontmatter from a markdown file
fn parse_frontmatter(content: &str) -> Option<String> {
    // Extract YAML frontmatter between --- markers
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

    let frontmatter = frontmatter_lines.join("\n");

    // Parse YAML
    match serde_yaml::from_str::<DeviceFrontmatter>(&frontmatter) {
        Ok(parsed) => Some(parsed.iroh_endpoint_id),
        Err(_) => None,
    }
}

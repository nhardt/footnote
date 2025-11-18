use anyhow::{Context, Result};
use iroh::{Endpoint, SecretKey};
use std::fs;

use super::{sync, vault};

const LOCAL_DEVICE_KEY_FILE: &str = "this_device";

/// Listen for incoming mirror connections
///
/// Starts an Iroh endpoint listening for sync connections from other devices.
/// Only accepts connections from devices belonging to the same user (verified via signatures).
pub async fn listen() -> Result<()> {
    // Load this device's Iroh secret key
    let keys_dir = vault::get_keys_dir()?;
    let key_file = keys_dir.join(LOCAL_DEVICE_KEY_FILE);
    let key_bytes = fs::read(&key_file)?;
    let key_array: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid key length"))?;
    let secret_key = SecretKey::from_bytes(&key_array);
    let endpoint_id = secret_key.public();

    // Get notes directory
    let notes_dir = vault::get_notes_dir()?;

    println!("\n📡 Mirror Sync - Listening");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Endpoint ID: {}", endpoint_id);
    println!("Ready to receive syncs from your other devices");
    println!("\nPress Ctrl+C to stop listening");
    println!();

    // Create Iroh endpoint with protocol handler
    let endpoint = Endpoint::builder()
        .secret_key(secret_key)
        .alpns(vec![sync::ALPN_MIRROR.to_vec()])
        .bind()
        .await?;

    // Accept connections in a loop
    loop {
        tokio::select! {
            Some(incoming) = endpoint.accept() => {
                let mut accepting = incoming.accept()?;
                let alpn = accepting.alpn().await?;
                let conn = accepting.await?;

                if alpn == sync::ALPN_MIRROR {
                    // Spawn a task to handle the connection
                    let notes_dir_clone = notes_dir.clone();
                    tokio::spawn(async move {
                        if let Err(e) = sync::handle_sync_accept(conn, &notes_dir_clone).await {
                            eprintln!("Error handling sync: {:?}", e);
                        }
                    });
                }
            }
            _ = tokio::signal::ctrl_c() => {
                println!("\nShutting down...");
                break;
            }
        }
    }

    Ok(())
}

/// Push mirror data to another device
///
/// For self-to-self sync (initial implementation):
/// - Must specify --device parameter
/// - Pushes all notes to the specified device (same user)
///
/// Future: --user parameter for user-to-user sharing
pub async fn push(user: Option<&str>, device: Option<&str>) -> Result<()> {
    match (user, device) {
        (None, Some(device_name)) => {
            // Self-to-self sync: push to specified device
            push_to_own_device(device_name).await
        }
        (None, None) => {
            anyhow::bail!(
                "Please specify a device to push to using --device\n\
                 Example: fieldnote mirror push --device laptop"
            );
        }
        (Some(_user_name), _) => {
            anyhow::bail!(
                "User-to-user sharing is not yet implemented.\n\
                 For now, only self-to-self sync is supported.\n\
                 Use: fieldnote mirror push --device <device_name>"
            );
        }
    }
}

/// Push to one of the user's own devices (self-to-self sync)
async fn push_to_own_device(device_name: &str) -> Result<()> {
    // Load this device's Iroh secret key
    let keys_dir = vault::get_keys_dir()?;
    let key_file = keys_dir.join(LOCAL_DEVICE_KEY_FILE);
    let key_bytes = fs::read(&key_file)?;
    let key_array: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid key length"))?;
    let secret_key = SecretKey::from_bytes(&key_array);

    // Look up the target device
    let devices_dir = vault::get_devices_dir()?;
    let device_file = devices_dir.join(format!("{}.md", device_name));

    if !device_file.exists() {
        anyhow::bail!(
            "Device '{}' not found.\n\
             Available devices can be seen with: fieldnote user read",
            device_name
        );
    }

    // Parse device file to get endpoint ID
    let device_content = fs::read_to_string(&device_file)?;
    let endpoint_id = parse_device_endpoint(&device_content)?;

    println!("\n📤 Mirror Sync - Push");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Target device: {}", device_name);
    println!("Target endpoint: {}", endpoint_id);
    println!();

    // Get notes directory
    let notes_dir = vault::get_notes_dir()?;

    // Push to the device
    sync::push_to_device(&notes_dir, endpoint_id, secret_key).await?;

    Ok(())
}

/// Parse the endpoint ID from device frontmatter
fn parse_device_endpoint(content: &str) -> Result<iroh::PublicKey> {
    // Simple frontmatter parsing
    if !content.starts_with("---\n") && !content.starts_with("---\r\n") {
        anyhow::bail!("Invalid device file format");
    }

    let rest = if content.starts_with("---\r\n") {
        &content[5..]
    } else {
        &content[4..]
    };

    let end_pos = rest
        .find("\n---\n")
        .or_else(|| rest.find("\r\n---\r\n"))
        .ok_or_else(|| anyhow::anyhow!("Frontmatter not properly closed"))?;

    let frontmatter = &rest[..end_pos];

    // Extract iroh_endpoint_id field
    for line in frontmatter.lines() {
        if let Some(stripped) = line.trim().strip_prefix("iroh_endpoint_id:") {
            let endpoint_str = stripped.trim().trim_matches('"');
            return endpoint_str
                .parse()
                .context("Invalid endpoint ID format");
        }
    }

    anyhow::bail!("iroh_endpoint_id not found in device file")
}

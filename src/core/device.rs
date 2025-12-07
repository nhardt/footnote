use crate::core::{crypto, vault};
use iroh::{Endpoint, SecretKey};
use n0_error::StdResultExt;
use serde::{Deserialize, Serialize};
use std::fs;
use tokio::sync::mpsc::{self, Receiver};
use uuid::Uuid;

const ALPN_DEVICE_AUTH: &[u8] = b"footnote/device-auth";
const MASTER_KEY_FILE: &str = "master_identity";
const LOCAL_DEVICE_KEY_FILE: &str = "this_device";

#[derive(Debug, Clone)]
pub enum DeviceAuthEvent {
    Listening { join_url: String },
    Connecting,
    ReceivedRequest { device_name: String },
    Verifying,
    Success { device_name: String },
    Error(String),
}

#[derive(Debug, Serialize, Deserialize)]
struct DeviceJoinRequest {
    device_name: String,
    iroh_endpoint_id: String,
    token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeviceJoinResponse {
    contact_json: String,
}

/// Check if the current device is a primary device
fn is_primary_device(vault_path: &std::path::Path) -> anyhow::Result<bool> {
    let footnotes_dir = vault_path.join(".footnotes");
    let master_key_file = footnotes_dir.join(MASTER_KEY_FILE);
    Ok(master_key_file.exists())
}

/// Get the local device name by matching the public key
pub fn get_local_device_name(vault_path: &std::path::Path) -> anyhow::Result<String> {
    let footnotes_dir = vault_path.join(".footnotes");
    let key_file = footnotes_dir.join(LOCAL_DEVICE_KEY_FILE);

    if !key_file.exists() {
        anyhow::bail!(
            "Local device key not found at {}. Run 'footnote init' first.",
            key_file.display()
        );
    }

    let key_bytes = fs::read(&key_file)?;
    let key_array: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid key length"))?;
    let secret_key = SecretKey::from_bytes(&key_array);
    let local_public_key = secret_key.public();

    // Read contact.json to find device name
    let contact_path = footnotes_dir.join("contact.json");
    let contact_content = fs::read_to_string(&contact_path)?;
    let contact_record: crypto::ContactRecord = serde_json::from_str(&contact_content)?;

    for device in &contact_record.devices {
        if device.iroh_endpoint_id == local_public_key.to_string() {
            return Ok(device.device_name.clone());
        }
    }

    anyhow::bail!(
        "Could not find device with matching public key in contact.json.\n\
        Local endpoint ID: {}",
        local_public_key
    )
}

/// Delete a device from the user's contact record
pub async fn delete_device(vault_path: &std::path::Path, device_name: &str) -> anyhow::Result<()> {
    // Load contact.json
    let footnotes_dir = vault_path.join(".footnotes");
    let contact_path = footnotes_dir.join("contact.json");
    let contact_content = fs::read_to_string(&contact_path)?;
    let mut contact_record: crypto::ContactRecord = serde_json::from_str(&contact_content)?;

    // Find and remove the device
    let original_count = contact_record.devices.len();
    contact_record.devices.retain(|d| d.device_name != device_name);

    // Check if device was found
    if contact_record.devices.len() == original_count {
        anyhow::bail!("Device '{}' not found in contact record", device_name);
    }

    // Update timestamp
    contact_record.updated_at = chrono::Utc::now().to_rfc3339();
    contact_record.signature = String::new();

    // Load master signing key
    let master_key_file = footnotes_dir.join(MASTER_KEY_FILE);
    if !master_key_file.exists() {
        anyhow::bail!("Master identity key not found");
    }

    let master_key_hex = fs::read_to_string(&master_key_file)?;
    let signing_key = crypto::signing_key_from_hex(&master_key_hex)?;

    // Re-sign contact record
    let signature = crypto::sign_contact_record(&contact_record, &signing_key)?;
    contact_record.signature = signature;

    // Save updated contact.json
    fs::write(
        &contact_path,
        serde_json::to_string_pretty(&contact_record)?,
    )?;

    Ok(())
}

/// Delete a device (legacy CLI interface)
pub async fn delete(user_name: &str, device_name: &str) -> anyhow::Result<()> {
    let _ = user_name; // Unused, kept for CLI compatibility
    let vault_path = vault::get_vault_path()?;
    delete_device(&vault_path, device_name).await
}

/// Create a new device (primary side) - generates join URL and listens for connection
pub async fn create_primary(vault_path: &std::path::Path) -> anyhow::Result<Receiver<DeviceAuthEvent>> {
    let (tx, rx) = mpsc::channel(32);
    // Check if this device is primary
    if !is_primary_device(vault_path)? {
        anyhow::bail!(
            "This device is not marked as primary. Only the primary device can create join URLs.\n\
            Run this command on your primary device."
        );
    }

    // Load master identity key
    let footnotes_dir = vault_path.join(".footnotes");
    let master_key_file = footnotes_dir.join(MASTER_KEY_FILE);
    if !master_key_file.exists() {
        anyhow::bail!("Master identity key not found. Run 'footnote init' first.");
    }

    let master_key_hex = fs::read_to_string(&master_key_file)?;
    let signing_key = crypto::signing_key_from_hex(&master_key_hex)?;

    // Generate one-time token
    let token = Uuid::new_v4().to_string();

    // Load this device's Iroh secret key to create endpoint
    let this_device_key_file = footnotes_dir.join(LOCAL_DEVICE_KEY_FILE);
    let key_bytes = fs::read(&this_device_key_file)?;
    let key_array: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid key length"))?;
    let secret_key = SecretKey::from_bytes(&key_array);

    // Create Iroh endpoint
    let endpoint = Endpoint::builder()
        .secret_key(secret_key.clone())
        .alpns(vec![ALPN_DEVICE_AUTH.to_vec()])
        .bind()
        .await?;

    let endpoint_id = secret_key.public();
    let join_url = format!("iroh://{}?token={}", endpoint_id, token);

    // Send initial listening event with URL
    let _ = tx.send(DeviceAuthEvent::Listening { join_url: join_url.clone() }).await;

    // Clone vault_path for use in spawned task
    let vault_path = vault_path.to_path_buf();

    // Spawn background task to handle connection
    tokio::spawn(async move {
        // Wait for connection
        if let Some(incoming) = endpoint.accept().await {
            let _ = tx.send(DeviceAuthEvent::Connecting).await;

            match async {
                let conn = incoming.accept()?.await?;
                let (mut send, mut recv) = conn.accept_bi().await.anyerr()?;

                // Read join request
                let request_bytes = recv.read_to_end(10000).await.anyerr()?;
                let request: DeviceJoinRequest = serde_json::from_slice(&request_bytes)?;

                let _ = tx.send(DeviceAuthEvent::ReceivedRequest {
                    device_name: request.device_name.clone()
                }).await;

                // Verify token
                if request.token != token {
                    anyhow::bail!("Invalid token. Authorization failed.");
                }

                let _ = tx.send(DeviceAuthEvent::Verifying).await;

                // Load current contact.json
                let contact_path = vault_path.join(".footnotes").join("contact.json");
                let contact_content = fs::read_to_string(&contact_path)?;
                let mut contact_record: crypto::ContactRecord = serde_json::from_str(&contact_content)?;

                // Add new device to contact record
                let new_device = crypto::ContactDevice {
                    device_name: request.device_name.clone(),
                    iroh_endpoint_id: request.iroh_endpoint_id,
                    added_at: chrono::Utc::now().to_rfc3339(),
                };

                contact_record.devices.push(new_device);
                contact_record.updated_at = chrono::Utc::now().to_rfc3339();
                contact_record.signature = String::new();

                // Re-sign entire contact record
                let signature = crypto::sign_contact_record(&contact_record, &signing_key)?;
                contact_record.signature = signature;

                // Save updated contact.json locally
                fs::write(
                    &contact_path,
                    serde_json::to_string_pretty(&contact_record)?,
                )?;

                // Send complete contact record to remote device
                let response = DeviceJoinResponse {
                    contact_json: serde_json::to_string(&contact_record)?,
                };

                let response_bytes = serde_json::to_vec(&response)?;
                send.write_all(&response_bytes).await.anyerr()?;
                send.finish().anyerr()?;

                conn.closed().await;

                Ok::<_, anyhow::Error>(request.device_name.clone())
            }.await {
                Ok(device_name) => {
                    let _ = tx.send(DeviceAuthEvent::Success { device_name }).await;
                }
                Err(e) => {
                    let _ = tx.send(DeviceAuthEvent::Error(e.to_string())).await;
                }
            }
        }
    });

    Ok(rx)
}

/// Create a new device (remote side) - joins using connection URL from primary
pub async fn create_remote(vault_path: &std::path::Path, connection_string: &str, device_name: &str) -> anyhow::Result<()> {
    // Check if vault already exists at the specified path
    let footnotes_check = vault_path.join(".footnotes");
    if footnotes_check.exists() {
        anyhow::bail!(
            "Vault already exists at {}. Remove it first if you want to join as a new device.",
            vault_path.display()
        );
    }

    // Parse connection string: iroh://endpoint-id?token=xyz
    let (endpoint_id, token) = parse_connection_string(connection_string)?;

    println!("\nDevice Join");
    println!("Connecting to primary device...");

    // Generate Iroh endpoint for this device
    let secret_key = SecretKey::generate(&mut rand::rng());
    let public_key = secret_key.public();

    // Create endpoint
    let endpoint = Endpoint::builder()
        .secret_key(secret_key.clone())
        .bind()
        .await?;

    // Connect to primary device
    let conn = endpoint.connect(endpoint_id, ALPN_DEVICE_AUTH).await?;
    let (mut send, mut recv) = conn.open_bi().await.anyerr()?;

    println!("Connected");

    // Send join request
    let request = DeviceJoinRequest {
        device_name: device_name.to_string(),
        iroh_endpoint_id: public_key.to_string(),
        token,
    };

    let request_bytes = serde_json::to_vec(&request)?;
    send.write_all(&request_bytes).await.anyerr()?;
    send.finish().anyerr()?;

    println!("Authenticating...");

    // Receive response
    let response_bytes = recv.read_to_end(100000).await.anyerr()?;
    let response: DeviceJoinResponse = serde_json::from_slice(&response_bytes)?;

    println!("Received contact record");

    // Parse and verify contact record
    let contact_record: crypto::ContactRecord = serde_json::from_str(&response.contact_json)?;

    if !crypto::verify_contact_signature(&contact_record)? {
        anyhow::bail!("Contact signature verification failed");
    }

    println!("Contact signature verified");

    // Create vault directory structure in current directory
    let footnotes_dir = vault_path.join(".footnotes");
    let contacts_dir = footnotes_dir.join("contacts");
    let trusted_sources_dir = vault_path.join("footnotes");

    fs::create_dir_all(&footnotes_dir)?;
    fs::create_dir_all(&contacts_dir)?;
    fs::create_dir_all(&trusted_sources_dir)?;

    // Store Iroh secret key
    let key_file = footnotes_dir.join(LOCAL_DEVICE_KEY_FILE);
    fs::write(&key_file, secret_key.to_bytes())?;

    // Store contact.json
    let contact_path = footnotes_dir.join("contact.json");
    fs::write(
        &contact_path,
        serde_json::to_string_pretty(&contact_record)?,
    )?;

    // Create device-specific home note at vault root
    let home_uuid = Uuid::new_v4();
    let home_filename = format!("home-{}.md", device_name);
    let home_file = vault_path.join(&home_filename);
    let home_content = format!(
        r#"---
uuid: {}
share_with: []
---

# Home ({})

Welcome to footnote on {}!
"#,
        home_uuid, device_name, device_name
    );
    fs::write(&home_file, home_content)?;

    println!("\nJoin complete!");
    println!("Identity: {}", contact_record.nickname);
    println!("Master key: {}", contact_record.master_public_key);
    println!("Device: {}", device_name);
    println!("Devices in contact: {}", contact_record.devices.len());
    println!("Vault created at: {}", vault_path.display());

    conn.close(0u8.into(), b"done");
    conn.closed().await;

    Ok(())
}

fn parse_connection_string(conn_str: &str) -> anyhow::Result<(iroh::PublicKey, String)> {
    // Expected format: iroh://endpoint-id?token=xyz
    let conn_str = conn_str.trim();

    if !conn_str.starts_with("iroh://") {
        anyhow::bail!("Invalid connection string. Expected format: iroh://endpoint-id?token=xyz");
    }

    let without_prefix = conn_str.strip_prefix("iroh://").unwrap();
    let parts: Vec<&str> = without_prefix.split('?').collect();

    if parts.len() != 2 {
        anyhow::bail!("Invalid connection string. Expected format: iroh://endpoint-id?token=xyz");
    }

    let endpoint_id: iroh::PublicKey = parts[0]
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid endpoint ID"))?;

    // Parse token from query string
    let query = parts[1];
    let token = query
        .split('&')
        .find(|s| s.starts_with("token="))
        .and_then(|s| s.strip_prefix("token="))
        .ok_or_else(|| anyhow::anyhow!("Token not found in connection string"))?
        .to_string();

    Ok((endpoint_id, token))
}

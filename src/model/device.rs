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

#[derive(Debug, Serialize, Deserialize)]
pub struct Device {
    pub name: String,
    pub iroh_endpoint_id: String,
    pub authorized_by: String, // hex-encoded verifying key
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContactDevice {
    pub device_name: String,
    pub iroh_endpoint_id: String,
    pub added_at: String,
}

#[derive(Debug, Clone)]
pub enum DeviceAuthEvent {
    Listening { join_url: String },
    Connecting,
    ReceivedRequest { device_name: String },
    Verifying,
    Success { device_name: String },
    Error(String),
}

/// Device record to be signed
#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceRecord {
    pub device_name: String,
    pub iroh_endpoint_id: String,
    pub authorized_by: String, // hex-encoded verifying key
    pub timestamp: String,
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
    contact_record
        .devices
        .retain(|d| d.device_name != device_name);

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
pub async fn create_primary(
    vault_path: &std::path::Path,
) -> anyhow::Result<Receiver<DeviceAuthEvent>> {
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
    let _ = tx
        .send(DeviceAuthEvent::Listening {
            join_url: join_url.clone(),
        })
        .await;

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

                let _ = tx
                    .send(DeviceAuthEvent::ReceivedRequest {
                        device_name: request.device_name.clone(),
                    })
                    .await;

                // Verify token
                if request.token != token {
                    anyhow::bail!("Invalid token. Authorization failed.");
                }

                let _ = tx.send(DeviceAuthEvent::Verifying).await;

                // Load current contact.json
                let contact_path = vault_path.join(".footnotes").join("contact.json");
                let contact_content = fs::read_to_string(&contact_path)?;
                let mut contact_record: crypto::ContactRecord =
                    serde_json::from_str(&contact_content)?;

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
            }
            .await
            {
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
pub async fn create_remote(
    vault_path: &std::path::Path,
    connection_string: &str,
    device_name: &str,
) -> anyhow::Result<()> {
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

async fn sync(vault_path: &std::path::Path, device_name: &str) -> Result<()> {
    // Load this device's Iroh secret key
    let footnotes_dir = vault_path.join(".footnotes");
    let key_file = footnotes_dir.join(LOCAL_DEVICE_KEY_FILE);
    let key_bytes = fs::read(&key_file)?;
    let key_array: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid key length"))?;
    let secret_key = SecretKey::from_bytes(&key_array);

    // Look up the target device in contact.json
    let contact_path = footnotes_dir.join("contact.json");
    let contact_content = fs::read_to_string(&contact_path)?;
    let contact_record: crate::core::crypto::ContactRecord =
        serde_json::from_str(&contact_content)?;

    let device = contact_record
        .devices
        .iter()
        .find(|d| d.device_name == device_name)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Device '{}' not found.\n\
                 Available devices can be seen with: footnote user read",
                device_name
            )
        })?;

    let endpoint_id = device.iroh_endpoint_id.parse::<iroh::PublicKey>()?;

    println!("\n📤 Mirror Sync - Push");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Target device: {}", device_name);
    println!("Target endpoint: {}", endpoint_id);
    println!();

    // Get notes directory
    let notes_dir = vault_path.to_path_buf();

    // Push to the device
    sync::push_to_device(&notes_dir, endpoint_id, secret_key).await?;

    Ok(())
}

/// Identify who a device belongs to
///
/// Returns either ("me", device_name) for same-user devices
/// or ("user", petname) for trusted user devices
pub async fn identify_device(
    vault_path: &Path,
    endpoint_id: &iroh::PublicKey,
) -> Result<(String, String)> {
    // First check if it's one of my devices
    let contact_path = vault_path.join(".footnotes").join("contact.json");
    let contact_content =
        fs::read_to_string(&contact_path).context("Failed to read contact.json")?;
    let contact_record: crypto::ContactRecord =
        serde_json::from_str(&contact_content).context("Failed to parse contact.json")?;

    // Search through my devices
    for device in &contact_record.devices {
        if let Ok(device_endpoint_id) = device.iroh_endpoint_id.parse::<iroh::PublicKey>() {
            if &device_endpoint_id == endpoint_id {
                return Ok(("me".to_string(), device.device_name.clone()));
            }
        }
    }

    // Check if it's a trusted user's device
    let contacts_dir = vault_path.join(".footnotes").join("contacts");
    if contacts_dir.exists() {
        for entry in fs::read_dir(&contacts_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                let petname = path.file_stem().unwrap().to_string_lossy().to_string();
                let content = fs::read_to_string(&path)?;

                if let Ok(user_contact) = serde_json::from_str::<crypto::ContactRecord>(&content) {
                    for device in &user_contact.devices {
                        if let Ok(device_endpoint_id) =
                            device.iroh_endpoint_id.parse::<iroh::PublicKey>()
                        {
                            if &device_endpoint_id == endpoint_id {
                                return Ok(("user".to_string(), petname.clone()));
                            }
                        }
                    }
                }
            }
        }
    }

    anyhow::bail!(
        "Device {} not found (not a known device or trusted user)",
        endpoint_id
    )
}

/// Handle an incoming sync connection
pub async fn handle_sync_accept(
    vault_path: &Path,
    connection: Connection,
    local_notes_dir: &Path,
) -> Result<()> {
    let remote_endpoint_id = connection.remote_id();

    // Identify the remote device (either same user or trusted user)
    let (device_type, identifier) = identify_device(vault_path, &remote_endpoint_id).await?;

    // Determine target directory based on device type
    let target_dir = if device_type == "me" {
        // Mirror sync from my own device -> notes/
        println!(
            "Receiving mirror sync from {} ({})",
            identifier, remote_endpoint_id
        );
        local_notes_dir.to_path_buf()
    } else {
        // Share from trusted user -> footnotes/{petname}/
        println!(
            "Receiving shared documents from {} ({})",
            identifier, remote_endpoint_id
        );
        let footnotes_dir = vault_path.join("footnotes").join(&identifier);
        fs::create_dir_all(&footnotes_dir)?;
        footnotes_dir
    };

    // Open bidirectional stream
    let (mut send, mut recv) = connection.accept_bi().await.anyerr()?;

    // Read manifest length (4 bytes, u32 big-endian)
    let mut len_buf = [0u8; 4];
    RecvStream::read_exact(&mut recv, &mut len_buf)
        .await
        .anyerr()?;
    let manifest_len = u32::from_be_bytes(len_buf) as usize;

    // Read and deserialize manifest
    let mut manifest_buf = vec![0u8; manifest_len];
    RecvStream::read_exact(&mut recv, &mut manifest_buf)
        .await
        .anyerr()?;
    let remote_manifest: manifest::Manifest =
        serde_json::from_slice(&manifest_buf).context("Failed to deserialize manifest")?;

    println!("Received manifest with {} files", remote_manifest.len());

    // Create local manifest
    let local_manifest =
        manifest::create_manifest(&target_dir).context("Failed to create local manifest")?;

    // Diff: find files that need to be synced
    let files_to_sync = manifest::diff_manifests(&local_manifest, &remote_manifest);

    println!("Requesting {} files", files_to_sync.len());

    // Request and receive each file
    for file_to_sync in &files_to_sync {
        // Send file request: path length (4 bytes) + path
        let path_str = file_to_sync
            .path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid path"))?;
        let path_bytes = path_str.as_bytes();
        let path_len = path_bytes.len() as u32;

        SendStream::write_all(&mut send, &path_len.to_be_bytes())
            .await
            .anyerr()?;
        SendStream::write_all(&mut send, path_bytes)
            .await
            .anyerr()?;

        // Receive file length (8 bytes, u64 big-endian)
        let mut file_len_buf = [0u8; 8];
        RecvStream::read_exact(&mut recv, &mut file_len_buf)
            .await
            .anyerr()?;
        let file_len = u64::from_be_bytes(file_len_buf) as usize;

        // Receive file contents
        let mut file_contents = vec![0u8; file_len];
        RecvStream::read_exact(&mut recv, &mut file_contents)
            .await
            .anyerr()?;

        // Write file to disk
        let full_path = target_dir.join(&file_to_sync.path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&full_path, file_contents)?;

        println!("  Synced: {} ({})", path_str, file_to_sync.reason_str());
    }

    // Send EOF signal (0-length path)
    SendStream::write_all(&mut send, &0u32.to_be_bytes())
        .await
        .anyerr()?;
    SendStream::finish(&mut send).anyerr()?;

    // Note: We do NOT delete local files (additive only, per design)

    println!("Sync complete! Received {} files", files_to_sync.len());
    connection.closed().await;

    Ok(())
}

/// Push files to a remote device
pub async fn push_to_device(
    local_notes_dir: &Path,
    remote_endpoint_id: iroh::PublicKey,
    local_secret_key: iroh::SecretKey,
) -> Result<()> {
    // Create manifest of local notes
    let manifest =
        manifest::create_manifest(local_notes_dir).context("Failed to create manifest")?;

    println!("Pushing {} files", manifest.len());

    // Create endpoint and connect
    let endpoint = iroh::Endpoint::builder()
        .secret_key(local_secret_key)
        .bind()
        .await?;

    let conn = endpoint
        .connect(remote_endpoint_id, ALPN_FOOTNOTE_FILES)
        .await
        .context("Failed to connect to remote device")?;

    let (mut send, mut recv) = conn.open_bi().await.anyerr()?;

    // Serialize and send manifest
    let encoded = serde_json::to_vec(&manifest).context("Failed to serialize manifest")?;
    let len = encoded.len() as u32;
    SendStream::write_all(&mut send, &len.to_be_bytes())
        .await
        .anyerr()?;
    SendStream::write_all(&mut send, &encoded).await.anyerr()?;

    println!("Manifest sent, waiting for file requests...");

    // Loop: read file requests and serve files
    let mut files_sent = 0;
    loop {
        // Read file path length
        let mut path_len_buf = [0u8; 4];
        RecvStream::read_exact(&mut recv, &mut path_len_buf)
            .await
            .anyerr()?;
        let path_len = u32::from_be_bytes(path_len_buf);

        // EOF signal (0-length path)
        if path_len == 0 {
            println!("Received EOF signal");
            break;
        }

        // Read file path
        let mut path_buf = vec![0u8; path_len as usize];
        RecvStream::read_exact(&mut recv, &mut path_buf)
            .await
            .anyerr()?;
        let file_path = String::from_utf8(path_buf).context("Invalid UTF-8 in file path")?;

        let path = PathBuf::from(&file_path);

        // Verify file is in manifest (by looking up UUID)
        let _entry = manifest
            .entries
            .values()
            .find(|e| e.path == path)
            .ok_or_else(|| anyhow::anyhow!("Requested file not in manifest: {}", file_path))?;

        // Read file from disk
        let full_path = local_notes_dir.join(&path);
        let file_contents = fs::read(&full_path)
            .with_context(|| format!("Failed to read file: {}", full_path.display()))?;

        // Send file length + contents
        let file_len = file_contents.len() as u64;
        SendStream::write_all(&mut send, &file_len.to_be_bytes())
            .await
            .anyerr()?;
        SendStream::write_all(&mut send, &file_contents)
            .await
            .anyerr()?;

        files_sent += 1;
        println!("✓ Sent: {} ({} bytes)", file_path, file_len);
    }

    SendStream::finish(&mut send).anyerr()?;
    conn.close(0u8.into(), b"done");
    conn.closed().await;

    println!("Push complete! Sent {} files", files_sent);

    Ok(())
}

/// Extension trait for FileToSync to provide string description
trait FileToSyncExt {
    fn reason_str(&self) -> &'static str;
}

impl FileToSyncExt for manifest::FileToSync {
    fn reason_str(&self) -> &'static str {
        match self.reason {
            manifest::SyncReason::NewFile => "new",
            manifest::SyncReason::UpdatedRemote => "updated",
        }
    }
}
/// Verify a device record signature
pub fn verify_device_signature(
    device_name: &str,
    iroh_endpoint_id: &str,
    authorized_by: &str,
    timestamp: &str,
    signature_hex: &str,
) -> anyhow::Result<bool> {
    // Reconstruct the device record
    let record = DeviceRecord {
        device_name: device_name.to_string(),
        iroh_endpoint_id: iroh_endpoint_id.to_string(),
        authorized_by: authorized_by.to_string(),
        timestamp: timestamp.to_string(),
    };

    // Serialize to get the same canonical representation
    let message = serde_yaml::to_string(&record)?;

    // Decode the signature
    let signature_bytes = hex::decode(signature_hex)?;
    let signature = Signature::from_slice(&signature_bytes)?;

    // Decode the verifying key
    let key_bytes = hex::decode(authorized_by)?;
    let verifying_key = VerifyingKey::from_bytes(
        &key_bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid key length"))?,
    )?;

    // Verify the signature
    match verifying_key.verify(message.as_bytes(), &signature) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Sign a device record with the master signing key
pub fn sign_device_record(
    device_name: &str,
    iroh_endpoint_id: &str,
    master_key: &SigningKey,
    timestamp: &str,
) -> anyhow::Result<String> {
    let verifying_key = master_key.verifying_key();
    let authorized_by = hex::encode(verifying_key.to_bytes());

    let record = DeviceRecord {
        device_name: device_name.to_string(),
        iroh_endpoint_id: iroh_endpoint_id.to_string(),
        authorized_by,
        timestamp: timestamp.to_string(),
    };

    // Serialize the record to create a canonical representation
    let message = serde_yaml::to_string(&record)?;

    // Sign the message
    let signature = master_key.sign(message.as_bytes());

    Ok(hex::encode(signature.to_bytes()))
}

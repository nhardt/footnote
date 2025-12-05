use anyhow::{Context, Result};
use iroh::endpoint::{Connection, RecvStream, SendStream};
use n0_error::StdResultExt;
use std::fs;
use std::path::{Path, PathBuf};

use super::{crypto, manifest, vault};

/// ALPN protocol identifier for mirror sync
pub const ALPN_MIRROR: &[u8] = b"footnote/mirror";

/// Identify who a device belongs to
///
/// Returns either ("me", device_name) for same-user devices
/// or ("user", petname) for trusted user devices
async fn identify_device(endpoint_id: &iroh::PublicKey) -> Result<(String, String)> {
    // First check if it's one of my devices
    let contact_path = vault::get_contact_path()?;
    let contact_content = fs::read_to_string(&contact_path)
        .context("Failed to read contact.json")?;
    let contact_record: crypto::ContactRecord = serde_json::from_str(&contact_content)
        .context("Failed to parse contact.json")?;

    // Search through my devices
    for device in &contact_record.devices {
        if let Ok(device_endpoint_id) = device.iroh_endpoint_id.parse::<iroh::PublicKey>() {
            if &device_endpoint_id == endpoint_id {
                return Ok(("me".to_string(), device.device_name.clone()));
            }
        }
    }

    // Check if it's a trusted user's device
    let contacts_dir = vault::get_contacts_dir()?;
    if contacts_dir.exists() {
        for entry in fs::read_dir(&contacts_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                let petname = path.file_stem().unwrap().to_string_lossy().to_string();
                let content = fs::read_to_string(&path)?;

                if let Ok(user_contact) = serde_json::from_str::<crypto::ContactRecord>(&content) {
                    for device in &user_contact.devices {
                        if let Ok(device_endpoint_id) = device.iroh_endpoint_id.parse::<iroh::PublicKey>() {
                            if &device_endpoint_id == endpoint_id {
                                return Ok(("user".to_string(), petname.clone()));
                            }
                        }
                    }
                }
            }
        }
    }

    anyhow::bail!("Device {} not found (not a known device or trusted user)", endpoint_id)
}

/// Handle an incoming sync connection
pub async fn handle_sync_accept(connection: Connection, local_notes_dir: &Path) -> Result<()> {
    let remote_endpoint_id = connection.remote_id();

    // Identify the remote device (either same user or trusted user)
    let (device_type, identifier) = identify_device(&remote_endpoint_id).await?;

    // Determine target directory based on device type
    let target_dir = if device_type == "me" {
        // Mirror sync from my own device -> notes/
        println!("Receiving mirror sync from {} ({})", identifier, remote_endpoint_id);
        local_notes_dir.to_path_buf()
    } else {
        // Share from trusted user -> footnotes/{petname}/
        println!("Receiving shared documents from {} ({})", identifier, remote_endpoint_id);
        let footnotes_dir = vault::get_trusted_user_dir(&identifier)?;
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
    let remote_manifest: manifest::Manifest = serde_json::from_slice(&manifest_buf)
        .context("Failed to deserialize manifest")?;

    println!(
        "Received manifest with {} files",
        remote_manifest.len()
    );

    // Create local manifest
    let local_manifest = manifest::create_manifest(&target_dir)
        .context("Failed to create local manifest")?;

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
    let manifest = manifest::create_manifest(local_notes_dir)
        .context("Failed to create manifest")?;

    println!("Pushing {} files", manifest.len());

    // Create endpoint and connect
    let endpoint = iroh::Endpoint::builder()
        .secret_key(local_secret_key)
        .bind()
        .await?;

    let conn = endpoint
        .connect(remote_endpoint_id, ALPN_MIRROR)
        .await
        .context("Failed to connect to remote device")?;

    let (mut send, mut recv) = conn.open_bi().await.anyerr()?;

    // Serialize and send manifest
    let encoded = serde_json::to_vec(&manifest).context("Failed to serialize manifest")?;
    let len = encoded.len() as u32;
    SendStream::write_all(&mut send, &len.to_be_bytes())
        .await
        .anyerr()?;
    SendStream::write_all(&mut send, &encoded)
        .await
        .anyerr()?;

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
        let file_path = String::from_utf8(path_buf)
            .context("Invalid UTF-8 in file path")?;

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

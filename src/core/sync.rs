use anyhow::{Context, Result};
use iroh::endpoint::{Connection, RecvStream, SendStream};
use n0_error::StdResultExt;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::{crypto, manifest, vault};

/// ALPN protocol identifier for mirror sync
pub const ALPN_MIRROR: &[u8] = b"fieldnote/mirror";

/// Device frontmatter loaded from device markdown files
#[derive(Debug, Deserialize)]
struct DeviceFrontmatter {
    device_name: String,
    iroh_endpoint_id: String,
    authorized_by: String,
    timestamp: String,
    signature: String,
}

/// Identity frontmatter loaded from identity.md
#[derive(Debug, Deserialize)]
struct IdentityFrontmatter {
    master_public_key: String,
    nickname: String,
}

/// Look up a device by its endpoint ID and verify it belongs to the same user
///
/// Returns the device name if found and verified, or an error
async fn verify_device_same_user(endpoint_id: &iroh::PublicKey) -> Result<String> {
    let devices_dir = vault::get_devices_dir()?;

    // Load our own master public key
    let identity_path = vault::get_identity_path()?;
    let identity_content = fs::read_to_string(&identity_path)
        .context("Failed to read identity.md")?;
    let local_identity: IdentityFrontmatter = parse_frontmatter(&identity_content)
        .context("Failed to parse local identity")?;

    // Search through all device files
    for entry in WalkDir::new(&devices_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        // Parse device frontmatter
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let device: DeviceFrontmatter = match parse_frontmatter(&content) {
            Ok(d) => d,
            Err(_) => continue,
        };

        // Check if this is the device we're looking for
        if device.iroh_endpoint_id == endpoint_id.to_string() {
            // Verify the device signature
            let signature_valid = crypto::verify_device_signature(
                &device.device_name,
                &device.iroh_endpoint_id,
                &device.authorized_by,
                &device.timestamp,
                &device.signature,
            )?;

            if !signature_valid {
                anyhow::bail!(
                    "Device signature verification failed for {}",
                    device.device_name
                );
            }

            // Verify the device was authorized by the same master key (same user)
            if device.authorized_by != local_identity.master_public_key {
                anyhow::bail!(
                    "Device {} belongs to a different user (authorized by {})",
                    device.device_name,
                    device.authorized_by
                );
            }

            return Ok(device.device_name);
        }
    }

    anyhow::bail!("Device not found for endpoint {}", endpoint_id)
}

/// Parse YAML frontmatter from markdown content
fn parse_frontmatter<T: for<'de> Deserialize<'de>>(content: &str) -> Result<T> {
    // Find frontmatter between --- delimiters
    if !content.starts_with("---\n") && !content.starts_with("---\r\n") {
        anyhow::bail!("No frontmatter found");
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

    let yaml_str = &rest[..end_pos];
    serde_yaml::from_str(yaml_str).context("Failed to parse YAML frontmatter")
}

/// Handle an incoming sync connection
pub async fn handle_sync_accept(connection: Connection, local_notes_dir: &Path) -> Result<()> {
    let remote_endpoint_id = connection.remote_id();

    // Verify the remote device belongs to the same user
    let remote_device_name = verify_device_same_user(&remote_endpoint_id).await?;

    println!(
        "Receiving sync from {} ({})",
        remote_device_name, remote_endpoint_id
    );

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
    let local_manifest = manifest::create_manifest(local_notes_dir)
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
        let full_path = local_notes_dir.join(&file_to_sync.path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&full_path, file_contents)?;

        println!("✓ Synced: {} ({})", path_str, file_to_sync.reason_str());
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
